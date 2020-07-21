use log::{debug, error};
use tokio_postgres::{Client, Error};

pub mod table;

pub use table::{ColSpec, Column, Table, TableSpec};

/// Database
pub struct DB {
    client: Client,
    tables: TableSpec,
}

impl DB {
    /// Create a new database given a reference to config
    /// and a table specification.
    /// If any tables are present in the database, will attempt to backup
    /// the data, clear the database, initialise it and fill it with the
    /// data that's been backed up.
    /// If `nobackup` is `true`, then will not backup/restore if tables are
    /// found - will just reset instead.
    pub async fn new(
        config: &tokio_postgres::Config,
        tables: TableSpec,
        nobackup: bool,
    ) -> Result<Self, Error> {
        // Connect
        let client = connect(config).await?;
        // The database object
        let db = Self { client, tables };
        // Attempt to initialise
        db.init(nobackup).await?;
        Ok(db)
    }
    /// Initialises the database.
    /// No tables - creates them and does nothing else.
    /// Attempts to backup-clear-init-restore if tables are found
    /// (unless `nobackup` is true in which case just clear-init).
    async fn init(&self, nobackup: bool) -> Result<(), Error> {
        // Empty database - only table creation required
        if self.is_empty().await? {
            self.create_all_tables().await?;
            return Ok(());
        }
        // Not empty - need to do backup-clear-init-restore
        if nobackup {
            self.drop_all_tables().await?;
            self.create_all_tables().await?;
            return Ok(());
        }
        self.backup().await?;
        self.drop_all_tables().await?;
        self.create_all_tables().await?;
        self.restore().await?;
        Ok(())
    }
    async fn backup(&self) -> Result<(), Error> {
        Ok(())
    }
    async fn restore(&self) -> Result<(), Error> {
        Ok(())
    }
    /// See if the database is empty (no tables)
    async fn is_empty(&self) -> Result<bool, Error> {
        let all_tables = self.get_all_table_names().await?;
        Ok(all_tables.is_empty())
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
    /// Drops all tables found in the database
    async fn drop_all_tables(&self) -> Result<(), Error> {
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
    /// Creates all stored tables
    async fn create_all_tables(&self) -> Result<(), Error> {
        self.create_tables(&self.tables).await?;
        Ok(())
    }
    /// Creates the given tables
    async fn create_tables(&self, tables: &[Table]) -> Result<(), Error> {
        for table in tables {
            self.create_table(table).await?;
        }
        Ok(())
    }
    /// Creates the given table
    async fn create_table(&self, table: &Table) -> Result<(), Error> {
        self.client
            .execute(table.construct_create_query().as_str(), &[])
            .await?;
        Ok(())
    }
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
