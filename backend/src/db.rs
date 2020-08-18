use crate::Result;

pub mod admin;
pub mod user;

pub type DBPool =
    mobc::Pool<mobc_postgres::PgConnectionManager<tokio_postgres::NoTls>>;
pub type DBCon =
    mobc::Connection<mobc_postgres::PgConnectionManager<tokio_postgres::NoTls>>;

const DB_POOL_MAX_OPEN: u64 = 32;
const DB_POOL_MAX_IDLE: u64 = 8;
const DB_POOL_TIMEOUT_SECONDS: u64 = 15;

pub fn create_pool(config: tokio_postgres::Config) -> Result<DBPool> {
    let manager =
        mobc_postgres::PgConnectionManager::new(config, tokio_postgres::NoTls);
    Ok(mobc::Pool::builder()
        .max_open(DB_POOL_MAX_OPEN)
        .max_idle(DB_POOL_MAX_IDLE)
        .get_timeout(Some(std::time::Duration::from_secs(
            DB_POOL_TIMEOUT_SECONDS,
        )))
        .build(manager))
}

/// Common database methods
#[async_trait::async_trait]
pub trait DB {
    /// Database name
    fn get_name(&self) -> &str;

    /// Client object
    fn get_pool(&self) -> &DBPool;

    /// Create all tables
    async fn create_all_tables(&self) -> Result<()>;

    /// Health check
    async fn health(&self) -> bool {
        self.get_con().await.is_ok()
    }

    /// Get a connection
    async fn get_con(&self) -> Result<DBCon> {
        let con = self.get_pool().get().await?;
        Ok(con)
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
        self.get_con()
            .await?
            .execute(
                format!("DROP TABLE IF EXISTS {} CASCADE;", all_tables)
                    .as_str(),
                &[],
            )
            .await?;
        Ok(())
    }

    /// Returns all found table names
    async fn get_all_table_names(&self) -> Result<Vec<String>> {
        // Vector of rows
        let all_tables = self
            .get_con()
            .await?
            .query(
                "SELECT tablename FROM pg_catalog.pg_tables \
                    WHERE schemaname = 'public';",
                &[],
            )
            .await?;
        // Transform into vector of strings
        let mut table_names: Vec<String> = Vec::with_capacity(all_tables.len());
        for row in all_tables {
            table_names.push(row.get::<usize, String>(0));
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
        pool: DBPool,
    }

    #[async_trait::async_trait]
    impl DB for TestDB {
        fn get_name(&self) -> &str {
            "test"
        }
        fn get_pool(&self) -> &DBPool {
            &self.pool
        }
        async fn create_all_tables(&self) -> Result<()> {
            let con = self.pool.get().await.unwrap();
            con.execute(
                "CREATE TABLE \"test_table\" \
                (\"test_field\" TEXT PRIMARY KEY)",
                &[],
            )
            .await
            .unwrap();
            con.execute(
                "CREATE TABLE \"test_table_2\" \
                (\"test_field\" TEXT PRIMARY KEY)",
                &[],
            )
            .await
            .unwrap();
            Ok(())
        }
    }

    #[tokio::test]
    pub async fn test_db() {
        crate::tests::setup_test_db("odcadmin_test_db").await;
        let test_db = TestDB {
            pool: create_pool(crate::tests::gen_test_config(
                "odcadmin_test_db",
            ))
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
        let con = test_db.pool.get().await.unwrap();
        con.execute("INSERT INTO test_table VALUES ('test')", &[])
            .await
            .unwrap();
        let test_table_content = con
            .query_opt("SELECT * FROM test_table", &[])
            .await
            .unwrap();
        assert!(test_table_content.is_some());
        test_db.reset().await.unwrap();
        let test_table_content = con
            .query_opt("SELECT * FROM test_table", &[])
            .await
            .unwrap();
        assert!(test_table_content.is_none());

        // Drop tables
        test_db.drop_all_tables().await.unwrap();
        assert!(test_db.is_empty().await.unwrap());
    }
}
