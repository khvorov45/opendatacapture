use log::{debug, error};
use tokio_postgres::{Client, Error, NoTls};

pub struct AdminDB {
    pub client: Client,
}

impl AdminDB {
    pub async fn connect(
        config: tokio_postgres::config::Config,
    ) -> Result<Self, Error> {
        // Connect
        let (client, connection) = config.connect(NoTls).await?;
        // Spawn off the connection
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("connection error: {}", e);
            }
        });
        // Return the database object
        let admindb = Self { client };
        Ok(admindb)
    }
    pub async fn state(&self) -> Result<DBState, Error> {
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
    pub async fn init(&self) -> Result<(), Error> {
        self.client
            .execute("CREATE TABLE admin (name TEXT);", &[])
            .await?;
        Ok(())
    }
}

pub enum DBState {
    Empty,
    Correct,
    Incorrect,
}
