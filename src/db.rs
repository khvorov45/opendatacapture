use log::{debug, error};
use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, Error};

pub mod table;

pub use table::{ColSpec, Column, Table, TableSpec};

/// Table data
#[derive(Debug)]
pub struct TableData {
    pub name: String,
    pub data: Vec<tokio_postgres::Row>,
}

impl TableData {
    /// New table data
    pub fn new(name: &str, data: Vec<tokio_postgres::Row>) -> Self {
        Self {
            name: String::from(name),
            data,
        }
    }
}

/// Table json
#[derive(Debug, Serialize, Deserialize)]
pub struct TableJson {
    pub name: String,
    pub json: serde_json::Value,
}

impl TableJson {
    pub fn new(name: &str, json: serde_json::Value) -> Self {
        Self {
            name: String::from(name),
            json,
        }
    }
}

/// Database
pub struct DB {
    client: Client,
    tables: TableSpec,
    backup_json_path: std::path::PathBuf,
}

impl DB {
    /// Create a new database given a reference to config
    /// and a table specification.
    /// If any tables are present in the database, will attempt to backup
    /// the data, clear the database, initialise it and fill it with the
    /// data that's been backed up.
    /// If `backup` is `false`, then will not backup/restore if tables are
    /// found - will just reset instead.
    pub async fn new(
        name: &str,
        config: &tokio_postgres::Config,
        tables: TableSpec,
        backup: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Connect
        let client = connect(config).await?;
        // The database object
        let db = Self {
            client,
            tables,
            backup_json_path: std::path::PathBuf::from(&format!(
                "backup-json/{}.json",
                name
            )),
        };
        // Attempt to initialise
        db.init(backup).await?;
        Ok(db)
    }
    /// Initialises the database.
    /// No tables - creates them and does nothing else.
    /// Attempts to backup-clear-init-restore if tables are found
    /// (unless `backup` is `false` in which case just clear-init).
    async fn init(
        &self,
        backup: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Empty database - only table creation required
        if self.is_empty().await? {
            debug!("initialising empty database");
            self.create_all_tables().await?;
            return Ok(());
        }
        // Not empty - need to do backup-clear-init-restore
        if !backup {
            debug!("initialising non-empty database with no backup");
            self.drop_all_tables().await?;
            self.create_all_tables().await?;
            return Ok(());
        }
        debug!("initialising non-empty database with backup");
        self.backup_json().await?;
        self.drop_all_tables().await?;
        self.create_all_tables().await?;
        if let Err(e) = self.restore_json().await {
            log::error!("failed to restore json: {}", e)
        }
        Ok(())
    }
    /// Backup in json format
    async fn backup_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("writing json backup to {:?}", self.backup_json_path);
        let all_json = self.get_all_json().await?;
        write_json(&all_json, self.backup_json_path.as_path())?;
        Ok(())
    }
    /// Restores data from json
    async fn restore_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("restoring json backup from {:?}", self.backup_json_path);
        let restored_json = read_json(self.backup_json_path.as_path())?;
        let tables_json: Vec<TableJson> =
            serde_json::from_value(restored_json)?;
        for table_json in tables_json {
            let this_table: &Table;
            match self.tables.iter().find(|t| t.name == table_json.name) {
                None => continue,
                Some(table) => this_table = table,
            }
            let table_rows: &Vec<serde_json::Value>;
            match table_json.json.as_array() {
                None => continue,
                Some(rows) => table_rows = rows,
            }
            for table_row in table_rows {
                let row_values: &serde_json::Map<String, serde_json::Value>;
                match table_row.as_object() {
                    None => continue,
                    Some(row) => row_values = row,
                }
                let query = this_table.construct_insert_query_json(row_values);
                self.client.execute(query.as_str(), &[]).await?;
            }
        }
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
        debug!("found table names: {:?}", table_names);
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
    /// Get all rows from the table
    pub async fn get_rows_data(
        &self,
        table_name: &str,
    ) -> Result<Vec<tokio_postgres::Row>, Error> {
        self.client
            .query(format!("SELECT * FROM {};", table_name).as_str(), &[])
            .await
    }
    /// Get all rows from the table
    pub async fn get_rows_json(
        &self,
        table_name: &str,
    ) -> Result<serde_json::Value, Error> {
        let all_rows_json = self
            .client
            .query(
                format!(
                    "SELECT ARRAY_TO_JSON(ARRAY_AGG(ROW_TO_JSON(\"{0}\"))) \
                    FROM \"{0}\";",
                    table_name
                )
                .as_str(),
                &[],
            )
            .await?;
        Ok(all_rows_json[0].get::<usize, serde_json::Value>(0))
    }
    /// Get one table's data
    pub async fn get_table_data(
        &self,
        table_name: &str,
    ) -> Result<TableData, Error> {
        let table_data = self.get_rows_data(table_name).await?;
        Ok(TableData::new(table_name, table_data))
    }
    /// Get one table's data
    pub async fn get_table_json(
        &self,
        table_name: &str,
    ) -> Result<TableJson, Error> {
        let table_json = self.get_rows_json(table_name).await?;
        Ok(TableJson::new(table_name, table_json))
    }
    /// Get all data out
    pub async fn get_all_data(&self) -> Result<Vec<TableData>, Error> {
        let mut table_data = Vec::with_capacity(self.tables.len());
        for table in &self.tables {
            let data = self.get_table_data(table.name.as_str()).await?;
            table_data.push(data);
        }
        Ok(table_data)
    }
    /// Get all data as json
    pub async fn get_all_json(
        &self,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let mut table_json = Vec::with_capacity(self.tables.len());
        for table in &self.tables {
            let json = self.get_table_json(table.name.as_str()).await?;
            table_json.push(json);
        }
        let table_json_ser = serde_json::to_value(&table_json)?;
        Ok(table_json_ser)
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

/// Write json
pub fn write_json(
    json: &serde_json::Value,
    filepath: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(filepath)?;
    serde_json::to_writer(&file, json)?;
    Ok(())
}

/// Read json
pub fn read_json(
    filepath: &std::path::Path,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(filepath)?;
    let reader = std::io::BufReader::new(file);
    let json = serde_json::from_reader(reader)?;
    Ok(json)
}

#[cfg(test)]
mod tests;
