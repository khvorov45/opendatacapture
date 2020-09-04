use crate::{Error, Result};
use sqlx::Row;

pub mod admin;
pub mod user;

use user::table::TableMeta;

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

#[async_trait::async_trait]
pub trait DB {
    /// Wrapped pool with accessible metadata
    fn get_pool_meta(&self) -> &PoolMeta;

    /// Reference to connection pool
    fn get_pool(&self) -> &Pool {
        self.get_pool_meta().get_pool()
    }

    /// Database name
    fn get_name(&self) -> &str {
        self.get_pool_meta().get_name()
    }

    /// Config used to connect to the database
    fn get_config(&self) -> ConnectionConfig {
        self.get_pool_meta().get_config()
    }

    /// Health check
    async fn health(&self) -> bool {
        self.get_pool().acquire().await.is_ok()
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

    async fn get_table_meta(&self, table_name: &str) -> Result<TableMeta> {
        Err(Error::TableNotPresent(table_name.to_string()))
    }

    /// See if the database is empty (no tables)
    async fn is_empty(&self) -> Result<bool> {
        let all_tables = self.get_all_table_names().await?;
        Ok(all_tables.is_empty())
    }

    /// Allows the execution of arbitrary SQL
    async fn execute(&self, sql: &str) -> Result<()> {
        sqlx::query(sql).execute(self.get_pool()).await?;
        Ok(())
    }
}

/// Pool, but you can actually pull metadata from it
#[derive(Debug)]
pub struct PoolMeta {
    pool: Pool,
    config: ConnectionConfig,
    name: String,
}

impl PoolMeta {
    /// Database name in config will be ignored
    pub async fn new(config: ConnectionConfig, name: &str) -> Result<Self> {
        let config = config.database(name);
        Ok(Self {
            pool: create_pool(config.clone()).await?,
            config,
            name: name.to_string(),
        })
    }
    /// Construction from opt
    pub async fn from_opt(opt: &crate::Opt) -> Result<Self> {
        let config = ConnectionConfig::from_opt(opt);
        Ok(Self {
            pool: create_pool(config.clone()).await?,
            config,
            name: opt.admindbname.to_string(),
        })
    }
    /// Reference to the actual connection pool
    pub fn get_pool(&self) -> &Pool {
        &self.pool
    }
    /// Database name
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    /// Config used to create the connection pool
    pub fn get_config(&self) -> ConnectionConfig {
        self.config.clone()
    }
}

impl FromOpt for ConnectionConfig {
    fn from_opt(opt: &crate::Opt) -> Self {
        if let Ok(url) = std::env::var("DATABASE_URL") {
            log::debug!("parsing DATABASE_URL: {}", url);
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
    log::debug!("creating pool with {:#?}", config);
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(DB_POOL_MAX_OPEN)
        .min_connections(DB_POOL_MAX_IDLE)
        .max_lifetime(std::time::Duration::from_secs(DB_POOL_TIMEOUT_SECONDS))
        .connect_with(config)
        .await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DB_NAME: &str = "odcadmin_test_db";

    struct TestDB {
        pool: PoolMeta,
    }

    impl DB for TestDB {
        fn get_pool_meta(&self) -> &PoolMeta {
            &self.pool
        }
    }

    impl TestDB {
        async fn reset(&self) -> Result<()> {
            log::info!("resetting \"{}\" database", self.get_name());
            self.drop_all_tables().await?;
            self.create_all_tables().await?;
            Ok(())
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
        crate::tests::setup_test_db(TEST_DB_NAME).await;
        let test_db = TestDB {
            pool: PoolMeta::new(
                crate::tests::gen_test_config("anything"),
                TEST_DB_NAME,
            )
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
