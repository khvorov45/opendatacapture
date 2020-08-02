use super::{json, password};

pub mod admin;
pub mod user;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type DBPool =
    mobc::Pool<mobc_postgres::PgConnectionManager<tokio_postgres::NoTls>>;
type DBCon =
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
    fn get_name(&self) -> String;

    /// Client object
    fn get_pool(&self) -> &DBPool;

    /// Create all tables
    async fn create_all_tables(&self) -> Result<()>;

    /// Get a connection
    async fn get_con(&self) -> Result<DBCon> {
        let con = self.get_pool().get().await?;
        Ok(con)
    }

    /// Drop all tables and re-create them
    async fn reset(&self) -> Result<()> {
        log::info!("resetting \"{}\" database with no backup", self.get_name());
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

pub mod error {
    /// Database errors
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        /// Represents all cases of `tokio_postgres::Error`
        #[error(transparent)]
        TokioPostgres(#[from] tokio_postgres::Error),
        /// Represents all cases of `argon2::Error`
        #[error(transparent)]
        Argon2(#[from] argon2::Error),
        /// Represents all cases of `json::Error`
        #[error(transparent)]
        Json(#[from] super::json::Error),
        /// Occurs when insert query cannot be constructed due to empty data
        #[error("want to address table {0} but it does not exist")]
        TableNotPresent(String),
        /// Occurs when a row cannot be parsed as map
        #[error("failed to parse as map: {0}")]
        RowParse(serde_json::Value),
        /// Occurs when insert query cannot be constructed due to empty data
        #[error("data to be inserted is empty")]
        InsertEmptyData,
        /// Occurs when addressing non-existent columns
        #[error("want to address columns {0:?} but they do not exist")]
        ColsNotPresent(Vec<String>),
        /// Pool error
        #[error("error getting connection from DB pool")]
        DBPool(#[from] mobc::Error<tokio_postgres::Error>),
    }
}
