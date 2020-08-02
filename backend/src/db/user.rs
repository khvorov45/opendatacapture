use crate::db::{create_pool, DBPool, DB};
use crate::{json, Error, Result};

pub mod table;

#[cfg(test)]
mod tests;

use table::{DBJson, RowJson, TableJson, TableMeta, TableSpec};

/// Database
pub struct UserDB {
    name: String,
    pool: DBPool,
    tables: TableSpec,
    backup_json_path: std::path::PathBuf,
}

#[async_trait::async_trait]
impl DB for UserDB {
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_pool(&self) -> &DBPool {
        &self.pool
    }
    /// Creates all stored tables
    async fn create_all_tables(&self) -> Result<()> {
        self.create_tables(&self.tables).await?;
        Ok(())
    }
}

impl UserDB {
    /// Create a new database given a reference to config
    /// and a table specification.
    /// If any tables are present in the database, will assume that they are
    /// correct.
    /// If empty, will create tables.
    pub async fn new(
        config: tokio_postgres::Config,
        tables: TableSpec,
    ) -> Result<Self> {
        // Connect
        let name = config.get_dbname().unwrap().to_string();
        let pool = create_pool(config)?;
        // The database object
        let db = Self {
            name: name.clone(),
            pool,
            tables,
            backup_json_path: std::path::PathBuf::from(&format!(
                "backup-json/{}.json",
                name
            )),
        };
        // Attempt to initialise
        db.init().await?;
        Ok(db)
    }
    /// Initialises the database.
    /// No tables - creates them.
    /// Some tables - does nothing (assumes that they are correct).
    async fn init(&self) -> Result<()> {
        // Empty database - table creation required
        if self.is_empty().await? {
            log::info!("initialising empty database \"{}\"", self.name);
            self.create_all_tables().await?;
        } else {
            log::info!(
                "found tables in database \"{}\", assuming they are correct",
                self.name
            )
        }
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
        self.get_con()
            .await?
            .execute(table.construct_create_query().as_str(), &[])
            .await?;
        Ok(())
    }
    /// Backup in json format
    pub async fn backup_json(&self) -> Result<()> {
        log::debug!("writing json backup to {:?}", self.backup_json_path);
        let db_json = self.get_db_json().await?;
        json::write(&db_json, self.backup_json_path.as_path())?;
        Ok(())
    }
    /// Restores data from json
    pub async fn restore_json(&self) -> Result<()> {
        log::debug!("restoring json backup from {:?}", self.backup_json_path);
        let tables_json: Vec<TableJson> =
            json::read(self.backup_json_path.as_path())?;
        for table_json in tables_json {
            self.find_table(table_json.name.as_str())?;
            if table_json.rows.is_empty() {
                log::info!("backup table \"{}\" is empty", table_json.name);
                continue;
            }
            log::info!(
                "restoring {} rows from \"{}\" table",
                table_json.rows.len(),
                table_json.name
            );
            self.insert(&table_json).await?
        }
        Ok(())
    }
    /// See if the database is empty (no tables)
    async fn is_empty(&self) -> Result<bool> {
        let all_tables = self.get_all_table_names().await?;
        Ok(all_tables.is_empty())
    }

    /// Find a table by name
    fn find_table(&self, name: &str) -> Result<&TableMeta> {
        match self.tables.iter().find(|t| t.name == name) {
            Some(t) => Ok(t),
            None => Err(Error::TableNotPresent(name.to_string())),
        }
    }

    /// Get all rows from the table.
    /// Collapse them into a map since we don't know the types in advance.
    /// This way every table has only one column of the same type.
    async fn get_rows_json(&self, table_name: &str) -> Result<Vec<RowJson>> {
        log::debug!("get json rows of table \"{}\"", table_name);
        let all_rows_json = self
            .get_con()
            .await?
            .query(
                self.find_table(table_name)?
                    .construct_select_json_query(&[], "")?
                    .as_str(),
                &[],
            )
            .await?;
        rows_to_json(all_rows_json)
    }
    /// Get one table's data
    async fn get_table_json(&self, table_name: &str) -> Result<TableJson> {
        let table_json = self.get_rows_json(table_name).await?;
        Ok(TableJson::new(table_name, table_json))
    }
    /// Get all data as json as per the specification
    async fn get_db_json(&self) -> Result<DBJson> {
        let mut db_json = Vec::with_capacity(self.tables.len());
        for table in &self.tables {
            let json = self.get_table_json(table.name.as_str()).await?;
            db_json.push(json);
        }
        Ok(db_json)
    }
    /// Insert data into a table
    pub async fn insert(&self, json: &TableJson) -> Result<()> {
        self.get_con()
            .await?
            .execute(
                self.find_table(json.name.as_str())?
                    .construct_insert_query(&json.rows)?
                    .as_str(),
                &[],
            )
            .await?;
        Ok(())
    }
    /// Select rows from a table
    pub async fn select(
        &self,
        name: &str,
        cols: &[&str],
        custom_post: &str,
        params: &[&(dyn tokio_postgres::types::ToSql + Sync)],
    ) -> Result<Vec<RowJson>> {
        log::debug!("select from table \"{}\"", name);
        let all_rows_json = self
            .get_con()
            .await?
            .query(
                self.find_table(name)?
                    .construct_select_json_query(cols, custom_post)?
                    .as_str(),
                params,
            )
            .await?;
        rows_to_json(all_rows_json)
    }
}

/// Converts tokio_postgres rows to json rows
fn rows_to_json(rows: Vec<tokio_postgres::Row>) -> Result<Vec<RowJson>> {
    let mut values: Vec<RowJson> = Vec::with_capacity(rows.len());
    if rows.is_empty() {
        return Ok(values);
    }
    for row in rows {
        let row_value: serde_json::Value = row.get(0);
        match row_value.as_object() {
            None => return Err(Error::RowParse(row_value)),
            Some(m) => values.push(m.clone()),
        }
    }
    Ok(values)
}
