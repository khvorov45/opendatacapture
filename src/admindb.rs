use log::{debug, error};
use tokio_postgres::{Client, Error};

use crate::error::APIError;

/// Administrative database
pub struct AdminDB {
    client: Client,
}

impl AdminDB {
    /// Create a new admin database given a reference to config.
    /// Checks that the structure is correct before returning.
    pub async fn new(
        config: &tokio_postgres::Config,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Connect
        let client = connect(config).await?;
        let db = Self { client };
        // Check state
        match db.state().await? {
            DBState::Empty => db.init().await?,
            DBState::Correct => (),
            DBState::Incorrect => {
                return Err(Box::new(APIError::new(
                    "Admin database has incorrect structure, \
                        clear or reset it before use",
                )))
            }
        }
        Ok(db)
    }
    /// Find out if the database is empty, corrently structured or
    /// incorrectly structured
    async fn state(&self) -> Result<DBState, Error> {
        // Vector of rows
        let all_tables = self
            .client
            .query(
                "SELECT tablename FROM pg_catalog.pg_tables \
                    WHERE schemaname = 'public';",
                &[],
            )
            .await?;
        if all_tables.is_empty() {
            debug!("database is empty");
            return Ok(DBState::Empty);
        }
        let expected_tables = ["admin"];
        if expected_tables.len() != all_tables.len() {
            debug!(
                "database structure incorrect because wrong number of tables: \
                    found {} while expected {}",
                all_tables.len(),
                expected_tables.len()
            );
            return Ok(DBState::Incorrect);
        }
        // Check that all tables present in the database are the required ones
        for row in all_tables {
            let tablename = row.get::<usize, &str>(0);
            if !expected_tables.contains(&tablename) {
                debug!(
                    "database stucture incorrect because table name \"{}\" \
                        was not expected",
                    tablename
                );
                return Ok(DBState::Incorrect);
            }
        }
        debug!("database structure correct");
        Ok(DBState::Correct)
    }
    /// Create the required database tables. Assumes the database is empty.
    async fn init(&self) -> Result<(), Error> {
        self.client
            .execute("CREATE TABLE admin (name TEXT);", &[])
            .await?;
        Ok(())
    }
}

/// Possible database states.
enum DBState {
    /// No tables
    Empty,
    /// All the correct tables
    Correct,
    /// Wrong tables or not enough tables
    Incorrect,
}

/// Creates a new connection
async fn connect(
    config: &tokio_postgres::Config,
) -> Result<tokio_postgres::Client, tokio_postgres::Error> {
    // Connect
    let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
    // Spawn off the connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });
    Ok(client)
}
