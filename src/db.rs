use tokio_postgres::Client;

use super::json;

pub mod error;
pub mod table;

pub use error::Error;
pub use table::{
    ColMeta, ColSpec, DBJson, RowJson, TableJson, TableMeta, TableSpec,
};

pub type Result<T> = std::result::Result<T, Error>;

/// Database
pub struct DB {
    name: String,
    client: Client,
    tables: TableSpec,
    backup_json_path: std::path::PathBuf,
    /// Whether the database was empty upon connection
    was_empty: bool,
}

impl DB {
    /// Create a new database given a reference to config
    /// and a table specification.
    /// If any tables are present in the database, will assume that they are
    /// correct.
    /// If empty, will create tables.
    pub async fn new(
        config: &tokio_postgres::Config,
        tables: TableSpec,
    ) -> Result<Self> {
        // Connect
        let client = connect(config).await?;
        let name = config.get_dbname().unwrap();
        // The database object
        let mut db = Self {
            name: String::from(name),
            client,
            tables,
            backup_json_path: std::path::PathBuf::from(&format!(
                "backup-json/{}.json",
                config.get_dbname().unwrap()
            )),
            was_empty: false, // Assume non-empty
        };
        // Attempt to initialise
        db.init().await?;
        Ok(db)
    }
    /// Whether the database was empty upon connection
    pub fn was_empty(&self) -> bool {
        self.was_empty
    }
    /// Initialises the database.
    /// No tables - creates them.
    /// Some tables - does nothing (assumes that they are correct).
    async fn init(&mut self) -> Result<()> {
        // Empty database - table creation required
        if self.is_empty().await? {
            log::info!("initialising empty database \"{}\"", self.name);
            self.create_all_tables().await?;
            self.was_empty = true; // Correct assumption
        } else {
            log::info!(
                "found tables in database \"{}\", assuming they are correct",
                self.name
            )
        }
        Ok(())
    }
    /// Drops all tables and recreates them. If `backup` is `true`, will
    /// attempt to do a json backup and restore.
    pub async fn reset(&self, backup: bool) -> Result<()> {
        // Not backing up - drop
        if !backup {
            log::info!("resetting \"{}\" database with no backup", self.name);
            self.drop_all_tables().await?;
            self.create_all_tables().await?;
            return Ok(());
        }
        log::info!("resetting \"{}\" database with backup", self.name);
        self.backup_json().await?;
        self.drop_all_tables().await?;
        self.create_all_tables().await?;
        if let Err(e) = self.restore_json().await {
            log::error!("failed to restore json: {}", e)
        }
        Ok(())
    }
    /// Backup in json format
    async fn backup_json(&self) -> Result<()> {
        log::debug!("writing json backup to {:?}", self.backup_json_path);
        let db_json = self.get_db_json().await?;
        json::write(&db_json, self.backup_json_path.as_path())?;
        Ok(())
    }
    /// Restores data from json
    async fn restore_json(&self) -> Result<()> {
        log::debug!("restoring json backup from {:?}", self.backup_json_path);
        let tables_json: Vec<TableJson> =
            json::read(self.backup_json_path.as_path())?;
        for table_json in tables_json {
            if self.find_table(table_json.name.as_str()).is_err() {
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
            if let Err(e) = self.insert(&table_json).await {
                log::error!(
                    "failed to restore table \"{}\": {}",
                    table_json.name,
                    e
                )
            }
        }
        Ok(())
    }
    /// See if the database is empty (no tables)
    async fn is_empty(&self) -> Result<bool> {
        let all_tables = self.get_all_table_names().await?;
        Ok(all_tables.is_empty())
    }
    /// Returns all current table names regardless of specification
    async fn get_all_table_names(&self) -> Result<Vec<String>> {
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
        log::debug!("found table names: {:?}", table_names);
        Ok(table_names)
    }
    /// Find a table by name
    fn find_table(&self, name: &str) -> Result<&TableMeta> {
        match self.tables.iter().find(|t| t.name == name) {
            Some(t) => Ok(t),
            None => Err(Error::TableNotPresent(name.to_string())),
        }
    }
    /// Drops all tables found in the database
    async fn drop_all_tables(&self) -> Result<()> {
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
    async fn drop_tables(&self, names: Vec<String>) -> Result<()> {
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
    async fn create_all_tables(&self) -> Result<()> {
        self.create_tables(&self.tables).await?;
        Ok(())
    }
    /// Creates the given tables
    async fn create_tables(&self, tables: &[TableMeta]) -> Result<()> {
        for table in tables {
            self.create_table(table).await?;
        }
        Ok(())
    }
    /// Creates the given table
    async fn create_table(&self, table: &TableMeta) -> Result<()> {
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
    ) -> Result<Vec<RowJson>> {
        log::debug!("get json rows of table \"{}\"", table_name);
        let all_rows_json = self
            .client
            .query(
                self.find_table(table_name)?
                    .construct_select_json_query(&[])?
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
            match row_value.as_object() {
                None => return Err(Error::RowParse(row_value)),
                Some(m) => values.push(m.clone()),
            }
        }
        Ok(values)
    }
    /// Get one table's data
    pub async fn get_table_json(&self, table_name: &str) -> Result<TableJson> {
        let table_json = self.get_rows_json(table_name).await?;
        Ok(TableJson::new(table_name, table_json))
    }
    /// Get all data as json as per the specification
    pub async fn get_db_json(&self) -> Result<DBJson> {
        let mut db_json = Vec::with_capacity(self.tables.len());
        for table in &self.tables {
            let json = self.get_table_json(table.name.as_str()).await?;
            db_json.push(json);
        }
        Ok(db_json)
    }
    /// Insert data into a table
    pub async fn insert(&self, json: &TableJson) -> Result<()> {
        self.client
            .execute(
                self.find_table(json.name.as_str())?
                    .construct_insert_query(&json.rows)?
                    .as_str(),
                &[],
            )
            .await?;
        Ok(())
    }
}

/// Creates a new connection
async fn connect(
    config: &tokio_postgres::Config,
) -> Result<tokio_postgres::Client> {
    // Connect
    let (client, connection) = config.connect(tokio_postgres::NoTls).await?;
    // Spawn off the connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            log::error!("connection error: {}", e);
        }
    });
    Ok(client)
}

#[cfg(test)]
mod tests;
