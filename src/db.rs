use log::{debug, error};
use tokio_postgres::{Client, Error};

use super::error;

pub mod table;

pub use table::{
    ColMeta, ColSpec, DBJson, RowJson, TableJson, TableMeta, TableSpec,
};

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
        let db_json = self.get_db_json().await?;
        write_json(&db_json, self.backup_json_path.as_path())?;
        Ok(())
    }
    /// Restores data from json
    async fn restore_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("restoring json backup from {:?}", self.backup_json_path);
        let tables_json: Vec<TableJson> =
            read_json(self.backup_json_path.as_path())?;
        for table_json in tables_json {
            if self.find_table(table_json.name.as_str()).is_none() {
                log::info!(
                    "table \"{}\" found it backup but not in database",
                    table_json.name
                );
                continue;
            }
            if table_json.rows.is_empty() {
                log::info!("backup table \"{}\" is empty", table_json.name);
                continue;
            }
            log::info!(
                "restoring {} rows from \"{}\" table",
                table_json.rows.len(),
                table_json.name
            );
            self.insert(&table_json).await?;
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
    /// Find a table by name
    fn find_table(&self, name: &str) -> Option<&TableMeta> {
        self.tables.iter().find(|t| t.name == name)
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
    async fn create_tables(&self, tables: &[TableMeta]) -> Result<(), Error> {
        for table in tables {
            self.create_table(table).await?;
        }
        Ok(())
    }
    /// Creates the given table
    async fn create_table(&self, table: &TableMeta) -> Result<(), Error> {
        self.client
            .execute(table.construct_create_query().as_str(), &[])
            .await?;
        Ok(())
    }
    /// Get all rows from the table.
    /// Collapse them into a map since we don't know the types in advance.
    /// This way every table has only one column of the same type.
    pub async fn get_rows_json(
        &self,
        table_name: &str,
    ) -> Result<Vec<RowJson>, Error> {
        let all_rows_json = self
            .client
            .query(
                format!(
                    "SELECT ROW_TO_JSON(\"{0}\") \
                    FROM \"{0}\";",
                    table_name
                )
                .as_str(),
                &[],
            )
            .await?;
        let mut values: Vec<RowJson> = Vec::with_capacity(all_rows_json.len());
        if all_rows_json.is_empty() {
            return Ok(values);
        }
        for row in all_rows_json {
            let row_value: serde_json::Value = row.get(0);
            let row_map;
            match row_value.as_object() {
                None => {
                    log::error!("cannot parse as map: {}", row_value);
                    row_map = serde_json::Map::new();
                }
                Some(m) => row_map = m.clone(),
            }
            values.push(row_map);
        }
        Ok(values)
    }
    /// Get one table's data
    pub async fn get_table_json(
        &self,
        table_name: &str,
    ) -> Result<TableJson, Error> {
        let table_json = self.get_rows_json(table_name).await?;
        Ok(TableJson::new(table_name, table_json))
    }
    /// Get all data as json
    pub async fn get_db_json(
        &self,
    ) -> Result<DBJson, Box<dyn std::error::Error>> {
        let mut db_json = Vec::with_capacity(self.tables.len());
        for table in &self.tables {
            let json = self.get_table_json(table.name.as_str()).await?;
            db_json.push(json);
        }
        Ok(db_json)
    }
    /// Insert data into a table
    pub async fn insert(
        &self,
        json: &TableJson,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Find the table
        if self.find_table(json.name.as_str()).is_none() {
            log::error!(
                "want to insert into table \"{}\" but it does not exist",
                json.name
            );
            return Ok(());
        }
        // Insert the data
        self.client
            .execute(json.construct_insert_query()?.as_str(), &[])
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

/// Write json
pub fn write_json<T: serde::Serialize>(
    json: T,
    filepath: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(filepath)?;
    serde_json::to_writer(&file, &serde_json::to_value(json)?)?;
    Ok(())
}

/// Read json
pub fn read_json<T: serde::de::DeserializeOwned>(
    filepath: &std::path::Path,
) -> Result<T, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(filepath)?;
    let reader = std::io::BufReader::new(file);
    let json: T = serde_json::from_reader(reader)?;
    Ok(json)
}

#[cfg(test)]
mod tests;
