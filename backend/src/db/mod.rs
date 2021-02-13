use crate::Result;
use sqlx::Row;

pub mod admin;
pub mod user;

const DB_POOL_MAX_OPEN: u32 = 32;
const DB_POOL_MAX_IDLE: u32 = 8;
const DB_POOL_TIMEOUT_SECONDS: u64 = 15;

type Database = sqlx::postgres::Postgres;
type Pool = sqlx::postgres::PgPool;
pub type ConnectionConfig = sqlx::postgres::PgConnectOptions;

#[derive(Debug)]
pub struct DB {
    pool: Pool,
    config: ConnectionConfig,
    name: String,
}

impl DB {
    /// Database name in config will be ignored
    async fn new(config: ConnectionConfig, name: &str) -> Result<Self> {
        let config = config.database(name);
        Ok(Self {
            pool: create_pool(config.clone()).await?,
            config,
            name: name.to_string(),
        })
    }

    /// Construction from opt
    async fn from_opt(opt: &crate::Opt) -> Result<Self> {
        let config = ConnectionConfig::from_opt(opt);
        let pool = create_pool(config.clone()).await?;
        let name = sqlx::query("SELECT current_database();")
            .fetch_one(&pool)
            .await?
            .get(0);
        Ok(Self { pool, config, name })
    }

    /// Reference to connection pool
    pub fn get_pool(&self) -> &Pool {
        &self.pool
    }

    /// Database name
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    /// Config used to connect to the database
    pub fn get_config(&self) -> &ConnectionConfig {
        &self.config
    }

    /// Health check
    async fn health(&self) -> bool {
        self.get_pool().acquire().await.is_ok()
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
}

trait FromOpt {
    fn from_opt(opt: &crate::Opt) -> Self;
}

impl FromOpt for ConnectionConfig {
    fn from_opt(opt: &crate::Opt) -> Self {
        log::debug!("parsing DATABASE_URL: {}", opt.database_url);
        match opt.database_url.parse() {
            Ok(o) => o,
            Err(e) => panic!("error parsing DATABASE_URL: {}", e),
        }
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

    const TEST_DB_NAME: &str = "postgres_test_db";

    #[tokio::test]
    pub async fn test_db() {
        let _ = pretty_env_logger::try_init();
        crate::tests::setup_test_db(TEST_DB_NAME).await;
        let test_db =
            DB::new(crate::tests::gen_test_config("anything"), TEST_DB_NAME)
                .await
                .unwrap();
        assert!(test_db.health().await);

        log::info!("table creation");

        assert!(test_db.get_all_table_names().await.unwrap().is_empty());
        sqlx::query(
            "CREATE TABLE \"test_table\" (\"test_field\" TEXT PRIMARY KEY)",
        )
        .execute(test_db.get_pool())
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE \"test_table_2\" \
                (\"test_field\" TEXT PRIMARY KEY)",
        )
        .execute(test_db.get_pool())
        .await
        .unwrap();

        assert_eq!(
            test_db.get_all_table_names().await.unwrap(),
            vec!["test_table", "test_table_2"]
        );

        // Remove test database -----------------------------------------------
        crate::tests::remove_test_db(&test_db).await;
    }
}
