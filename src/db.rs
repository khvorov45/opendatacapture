use log::{debug, error};
use std::collections::HashMap;
use tokio_postgres::{Client, Error};

use crate::error::APIError;

pub mod constants;
pub mod table;

pub use table::ColSpec;

pub type TableSpec = HashMap<String, ColSpec>;

/// Administrative database
pub struct DB {
    client: Client,
    tables: TableSpec,
}

impl DB {
    /// Create a new database given a reference to config.
    /// Checks that the structure is correct before returning.
    /// Checks that the tables are correct before returning.
    pub async fn new(
        config: &tokio_postgres::Config,
        tables: TableSpec,
        forcereset: bool,
        forcetables: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Connect
        let client = connect(config).await?;
        // The database object
        let db = Self { client, tables };
        // Check state
        match db.state().await? {
            DBState::Empty => db.init().await?,
            DBState::Correct => {
                let incorrect_tables = db.find_incorrect_tables().await?;
                if !incorrect_tables.is_empty() {
                    if !forcetables {
                        return Err(Box::new(APIError::new(
                            format!(
                                "Database has incorrect tables: {}. \
                            Run with option \
                            --forcetables to cascade reset them.",
                                incorrect_tables.join(", ")
                            )
                            .as_str(),
                        )));
                    }
                    db.reset_tables(incorrect_tables).await?;
                }
            }
            DBState::Incorrect => {
                if forcereset {
                    db.reset().await?
                } else {
                    return Err(Box::new(APIError::new(
                        "Database has incorrect structure. \
                        Clear or reset it before use. Run with option \
                        --forcereset to do this automatically.",
                    )));
                }
            }
        }
        Ok(db)
    }
    /// Find out if the database is empty, correctly structured or
    /// incorrectly structured
    async fn state(&self) -> Result<DBState, Error> {
        let all_tables = self.get_all_table_names().await?;
        // Empty
        if all_tables.is_empty() {
            debug!("database is empty");
            return Ok(DBState::Empty);
        }
        // Wrong table amount
        if self.tables.len() != all_tables.len() {
            debug!(
                "database structure incorrect because wrong number of tables: \
                    found {} while expected {}",
                all_tables.len(),
                self.tables.len()
            );
            return Ok(DBState::Incorrect);
        }
        let expected_tables: Vec<&String> = self.tables.keys().collect();
        // Check that all tables present in the database are the required ones
        for tablename in all_tables {
            if !expected_tables.contains(&&tablename) {
                debug!(
                    "database structure incorrect because table name \"{}\" \
                        was not expected",
                    tablename
                );
                return Ok(DBState::Incorrect);
            }
        }
        debug!("database structure correct");
        Ok(DBState::Correct)
    }

    /// Returns all current table names regardless of database correctness
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
        debug!("Found table names: {:?}", table_names);
        Ok(table_names)
    }
    /// Create the required database tables. Assumes the database is empty.
    async fn init(&self) -> Result<(), Error> {
        for (name, cols) in &self.tables {
            self.client
                .execute(
                    table::construct_create_query(name, cols).as_str(),
                    &[],
                )
                .await?;
        }
        Ok(())
    }
    /// Clear followed by init
    async fn reset(&self) -> Result<(), Error> {
        self.clear().await?;
        self.init().await
    }
    /// Drops all tables regardles of the correctness of the database
    async fn clear(&self) -> Result<(), Error> {
        let all_tables: Vec<String> = self
            // Vector of strings
            .get_all_table_names()
            .await?;
        if all_tables.is_empty() {
            return Ok(());
        }
        self.drop_tables(all_tables).await?;
        Ok(())
    }
    /// Checks all tables for correctness. Assumes the database is correctly
    /// structured.
    async fn find_incorrect_tables(&self) -> Result<Vec<String>, Error> {
        let mut incorrect_tables = Vec::new();
        for (name, cols) in &self.tables {
            if !self.verify_table(name, cols).await? {
                incorrect_tables.push(String::from(name));
            }
        }
        debug!("Found incorrect tables: {:?}", incorrect_tables);
        Ok(incorrect_tables)
    }
    /// Checks if one table is correct
    async fn verify_table(
        &self,
        name: &str,
        cols_expected: &ColSpec,
    ) -> Result<bool, Error> {
        // Pull all names and types
        let names_and_types_got = self
            .client
            .query(
                "SELECT column_name, data_type \
                FROM information_schema.columns \
                WHERE table_name = $1",
                &[&name],
            )
            .await?;
        let mut cols_obtained = ColSpec::new();
        for row in &names_and_types_got {
            cols_obtained.insert(row.get(0), row.get(1));
        }
        Ok(table::verify(name, &cols_obtained, cols_expected))
    }
    /// Resets the given tables
    async fn reset_tables(&self, names: Vec<String>) -> Result<(), Error> {
        self.drop_tables(names).await?;
        // Need init because may have cascade-dropped more tables than passed
        self.init().await?;
        Ok(())
    }
    /// Drops the given tables
    async fn drop_tables(&self, names: Vec<String>) -> Result<(), Error> {
        let all_tables: String = names
            // Surround by quotation marks
            .iter()
            .map(|name| format!("\"{}\"", name))
            .collect::<Vec<String>>()
            // Join into a comma-separated string
            .join(",");
        self.client
            .execute(
                format!("DROP TABLE IF EXISTS {} CASCADE;", all_tables)
                    .as_str(),
                &[],
            )
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
mod tests;
