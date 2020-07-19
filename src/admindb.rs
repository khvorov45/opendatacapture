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
        forcereset: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Connect
        let client = connect(config).await?;
        let db = Self { client };
        // Check state
        match db.state().await? {
            DBState::Empty => db.init().await?,
            DBState::Correct => (), // @TODO table-level checking
            DBState::Incorrect => {
                if forcereset {
                    db.reset().await?
                } else {
                    return Err(Box::new(APIError::new(
                        "Admin database has incorrect structure. \
                        Clear or reset it before use. Run with option \
                        --forcereset to do this automatically.",
                    )));
                }
            }
        }
        Ok(db)
    }
    /// Find out if the database is empty, corrently structured or
    /// incorrectly structured
    async fn state(&self) -> Result<DBState, Error> {
        let all_tables = self.get_all_table_names().await?;
        // Empty
        if all_tables.is_empty() {
            debug!("database is empty");
            return Ok(DBState::Empty);
        }
        let expected_tables = ["admin"];
        // Wrong table amount
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
        for tablename in all_tables {
            if !expected_tables.contains(&tablename.as_str()) {
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

    async fn get_all_table_names(&self) -> Result<Vec<String>, Error> {
        // Vector of rows
        let all_tables = self
            .client
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
        Ok(table_names)
    }
    /// Create the required database tables. Assumes the database is empty.
    async fn init(&self) -> Result<(), Error> {
        self.client
            .execute("CREATE TABLE admin (name TEXT);", &[])
            .await?;
        Ok(())
    }
    /// Clear followed by init
    async fn reset(&self) -> Result<(), Error> {
        self.clear().await?;
        self.init().await
    }
    /// Drops all tables
    async fn clear(&self) -> Result<(), Error> {
        let all_tables: String = self
            // Vector of strings
            .get_all_table_names()
            .await?
            // Surround by quotation marks
            .iter()
            .map(|name| format!("\"{}\"", name))
            .collect::<Vec<String>>()
            // Join into a comma-separated string
            .join(",");
        if all_tables.is_empty() {
            return Ok(());
        }
        self.client
            .execute(format!("DROP TABLE {};", all_tables).as_str(), &[])
            .await?;
        Ok(())
    }
}

/// Possible database states.
#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    // Assume that ther is a database called odctest, connect with the same user
    // and password
    fn get_test_config() -> tokio_postgres::Config {
        let mut dbconfig = tokio_postgres::Config::new();
        dbconfig
            .host("localhost")
            .port(5432)
            .dbname("odctest")
            .user("odcapi")
            .password("odcapi");
        dbconfig
    }

    #[tokio::test]
    async fn test_db_creation() {
        // Connect to test database
        let db = AdminDB {
            client: connect(&get_test_config()).await.unwrap(),
        };
        // Clear
        db.clear().await.unwrap();
        let all_tables = db.get_all_table_names().await.unwrap();
        assert!(all_tables.is_empty());
        assert_eq!(db.state().await.unwrap(), DBState::Empty);
        // Initialise
        db.init().await.unwrap();
        let all_tables = db.get_all_table_names().await.unwrap();
        assert!(!all_tables.is_empty());
        assert_eq!(db.state().await.unwrap(), DBState::Correct);
        db.reset().await.unwrap();
        assert_eq!(db.state().await.unwrap(), DBState::Correct);
        // Insert an extra table
        db.client
            .execute("CREATE TABLE extratable (name TEXT);", &[])
            .await
            .unwrap();
        // See if the incorrect state is detected
        assert_eq!(db.state().await.unwrap(), DBState::Incorrect);
        // Reset
        db.reset().await.unwrap();
        assert_eq!(db.state().await.unwrap(), DBState::Correct);
    }
}
