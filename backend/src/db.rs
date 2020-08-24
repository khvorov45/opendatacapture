use crate::Result;
use sqlx::Row;

pub mod admin;
pub mod user;

const DB_POOL_MAX_OPEN: u32 = 32;
const DB_POOL_MAX_IDLE: u32 = 8;
const DB_POOL_TIMEOUT_SECONDS: u64 = 15;

pub type Database = sqlx::postgres::Postgres;
pub type DBRow = sqlx::postgres::PgRow;
pub type Pool = sqlx::postgres::PgPool;
pub type Connection = sqlx::pool::PoolConnection<Database>;
pub type ConnectionConfig = sqlx::postgres::PgConnectOptions;

pub trait FromOpt {
    fn from_opt(opt: &crate::Opt) -> Self;
}

impl FromOpt for ConnectionConfig {
    fn from_opt(opt: &crate::Opt) -> Self {
        if let Ok(url) = std::env::var("DATABASE_URL") {
            match url.parse() {
                Ok(o) => return o,
                Err(e) => log::error!(
                    "error parsing DATABASE_URL, fall back to args: {}",
                    e
                ),
            }
        }
        Self::new()
            .host(opt.dbhost.as_str())
            .port(opt.dbport)
            .database(opt.admindbname.as_str())
            .username(opt.apiusername.as_str())
            .password(opt.apiuserpassword.as_str())
    }
}

async fn create_pool(config: ConnectionConfig) -> Result<Pool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(DB_POOL_MAX_OPEN)
        .min_connections(DB_POOL_MAX_IDLE)
        .max_lifetime(std::time::Duration::from_secs(DB_POOL_TIMEOUT_SECONDS))
        .connect_with(config)
        .await?;
    Ok(pool)
}

/// Common database methods
#[async_trait::async_trait]
pub trait DB {
    /// Database name
    fn get_name(&self) -> &str;

    /// Client object
    fn get_pool(&self) -> &Pool;

    /// Create all tables
    async fn create_all_tables(&self) -> Result<()>;

    /// Health check
    async fn health(&self) -> bool {
        self.get_pool().acquire().await.is_ok()
    }

    /// Drop all tables and re-create them
    async fn reset(&self) -> Result<()> {
        log::info!("resetting \"{}\" database", self.get_name());
        self.drop_all_tables().await?;
        self.create_all_tables().await?;
        Ok(())
    }

    /// Drop all tables found in the database
    async fn drop_all_tables(&self) -> Result<()> {
        let all_tables: Vec<String> = self
            // Vector of strings
            .get_all_table_names()
            .await?;
        self.drop_tables(all_tables).await?;
        Ok(())
    }

    /// Drops the given tables
    async fn drop_tables(&self, names: Vec<String>) -> Result<()> {
        if names.is_empty() {
            return Ok(());
        }
        let all_tables: String = names
            // Surround by quotation marks
            .iter()
            .map(|name| format!("\"{}\"", name))
            .collect::<Vec<String>>()
            // Join into a comma-separated string
            .join(",");
        sqlx::query(
            format!("DROP TABLE IF EXISTS {} CASCADE;", all_tables).as_str(),
        )
        .execute(self.get_pool())
        .await?;
        Ok(())
    }

    /// Returns all found table names
    async fn get_all_table_names(&self) -> Result<Vec<String>> {
        // Vector of rows
        let res = sqlx::query(
            "SELECT tablename FROM pg_catalog.pg_tables \
            WHERE schemaname = 'public';",
        )
        .fetch_all(self.get_pool())
        .await?;
        let mut table_names = Vec::<String>::with_capacity(res.len());
        for row in res {
            table_names.push(row.get(0));
        }
        log::debug!(
            "found table names: {:?} in database {}",
            table_names,
            self.get_name()
        );
        Ok(table_names)
    }

    /// See if the database is empty (no tables)
    async fn is_empty(&self) -> Result<bool> {
        let all_tables = self.get_all_table_names().await?;
        Ok(all_tables.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestDB {
        pool: Pool,
    }

    #[async_trait::async_trait]
    impl DB for TestDB {
        fn get_name(&self) -> &str {
            "test"
        }
        fn get_pool(&self) -> &Pool {
            &self.pool
        }
        async fn create_all_tables(&self) -> Result<()> {
            sqlx::query(
                "CREATE TABLE \"test_table\" (\"test_field\" TEXT PRIMARY KEY)",
            )
            .execute(self.get_pool())
            .await
            .unwrap();
            sqlx::query(
                "CREATE TABLE \"test_table_2\" \
                (\"test_field\" TEXT PRIMARY KEY)",
            )
            .execute(self.get_pool())
            .await
            .unwrap();
            Ok(())
        }
    }

    #[tokio::test]
    pub async fn test_db() {
        let _ = pretty_env_logger::try_init();
        crate::tests::setup_test_db("odcadmin_test_db").await;
        let test_db = TestDB {
            pool: create_pool(crate::tests::gen_test_config(
                "odcadmin_test_db",
            ))
            .await
            .unwrap(),
        };
        assert!(test_db.health().await);

        // table creation
        assert!(test_db.is_empty().await.unwrap());
        test_db.create_all_tables().await.unwrap();
        assert_eq!(
            test_db.get_all_table_names().await.unwrap(),
            vec!["test_table", "test_table_2"]
        );
        assert!(!test_db.is_empty().await.unwrap());

        // Test reset
        sqlx::query("INSERT INTO test_table VALUES ('test')")
            .execute(test_db.get_pool())
            .await
            .unwrap();
        let test_table_content = sqlx::query("SELECT * FROM test_table")
            .fetch_all(test_db.get_pool())
            .await
            .unwrap();
        assert_eq!(test_table_content.len(), 1);
        test_db.reset().await.unwrap();
        let test_table_content = sqlx::query("SELECT * FROM test_table")
            .fetch_all(test_db.get_pool())
            .await
            .unwrap();
        assert!(test_table_content.is_empty());

        // Drop tables
        test_db.drop_all_tables().await.unwrap();
        assert!(test_db.is_empty().await.unwrap());
    }
}
