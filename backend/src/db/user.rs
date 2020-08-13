use crate::db::{create_pool, DBPool, DB};
use crate::{json, Error, Result};

pub mod table;

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
    fn get_name(&self) -> &str {
        self.name.as_str()
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

#[cfg(test)]
mod tests {
    use super::table::{ColMeta, ColSpec};
    use super::*;

    // Test config
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

    // Makes sure odctest database exists.
    // Assumes odcadmin database exists
    async fn setup_odctest() {
        let mut config = get_test_config();
        config.dbname("odcadmin");
        let (odcadmin_client, con) =
            config.connect(tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(async move {
            con.await.unwrap();
        });
        odcadmin_client
            .execute("DROP DATABASE IF EXISTS odctest", &[])
            .await
            .unwrap();
        odcadmin_client
            .execute("CREATE DATABASE odctest", &[])
            .await
            .unwrap();
    }

    // Test primary table
    fn get_test_primary_table() -> TableMeta {
        let mut cols = ColSpec::new();
        cols.push(ColMeta::new("id", "SERIAL", "PRIMARY KEY"));
        cols.push(ColMeta::new("email", "TEXT", "NOT NULL"));
        TableMeta::new("primary", cols, "")
    }

    // One entry for the primary table
    fn get_primary_entry_from_json(json: &str) -> RowJson {
        serde_json::from_str::<RowJson>(json).unwrap()
    }

    // Some data for the primary table
    fn get_primary_sample_data() -> TableJson {
        let mut sample_data = Vec::new();
        sample_data.push(get_primary_entry_from_json(
            r#"{"email": "test1@example.com"}"#,
        ));
        sample_data.push(get_primary_entry_from_json(
            r#"{"email": "test2@example.com"}"#,
        ));
        TableJson::new("primary", sample_data)
    }

    // Test secondary table
    fn get_test_secondary_table() -> TableMeta {
        let mut cols = ColSpec::new();
        cols.push(ColMeta::new("id", "INTEGER", ""));
        cols.push(ColMeta::new("timepoint", "INTEGER", ""));
        TableMeta::new(
            "secondary",
            cols,
            "PRIMARY KEY(\"id\", \"timepoint\"),\
        FOREIGN KEY(\"id\") REFERENCES \"primary\"(\"id\")",
        )
    }

    // Some data for the secondary table
    fn get_secondary_sample_data() -> TableJson {
        let mut sample_data = Vec::new();
        sample_data
            .push(get_primary_entry_from_json(r#"{"id": 1, "timepoint": 1}"#));
        sample_data
            .push(get_primary_entry_from_json(r#"{"id": 1, "timepoint": 2}"#));
        TableJson::new("secondary", sample_data)
    }

    // Test database specification
    fn get_testdb_spec() -> TableSpec {
        let mut test_tables = TableSpec::new();
        test_tables.push(get_test_primary_table());
        test_tables.push(get_test_secondary_table());
        test_tables
    }

    // A different table specification
    fn get_testdb_spec_alt() -> TableSpec {
        let mut table_spec = get_testdb_spec();
        // Add an extra table
        let mut extra_cols = Vec::new();
        extra_cols.push(ColMeta::new("id", "SERIAL", "PRIMARY KEY"));
        table_spec.push(TableMeta::new("extra", extra_cols, ""));
        // Remove a table
        table_spec.retain(|t| t.name != "secondary");
        table_spec
    }

    // Inserts data into the database
    async fn insert_test_data(db: &UserDB) {
        test_rows_absent(db).await;
        let primary_data = get_primary_sample_data();
        db.insert(&primary_data).await.unwrap();
        let secondary_data = get_secondary_sample_data();
        db.insert(&secondary_data).await.unwrap();
        test_rows_present(db).await;
    }

    // Whether the sample data is present
    async fn test_rows_absent(db: &UserDB) {
        assert!(db.get_rows_json("primary").await.unwrap().is_empty());
        assert!(db.get_rows_json("secondary").await.unwrap().is_empty());
    }

    // Whether the sample data is absent
    async fn test_rows_present(db: &UserDB) {
        assert_eq!(
            db.get_rows_json("primary").await.unwrap().len(),
            get_primary_sample_data().rows.len()
        );
        assert_eq!(
            db.get_rows_json("secondary").await.unwrap().len(),
            get_secondary_sample_data().rows.len()
        );
    }

    // Test database
    #[tokio::test]
    async fn user_db() {
        let _ = pretty_env_logger::try_init();
        setup_odctest().await;
        let test_config = get_test_config();
        // Manually created database object - use to control what database we
        // are connecting to
        let db = UserDB {
            name: String::from("odctest"),
            backup_json_path: std::path::PathBuf::from(
                "backup-json/odctest.json",
            ),
            pool: create_pool(test_config.clone()).unwrap(),
            tables: get_testdb_spec(),
        };

        // Make sure there are no tables
        db.drop_all_tables().await.unwrap();
        assert!(db.is_empty().await.unwrap());

        // Connect to empty
        let new_db = UserDB::new(test_config.clone(), get_testdb_spec())
            .await
            .unwrap();
        test_rows_absent(&new_db).await;

        // Recreate the tables
        db.create_all_tables().await.unwrap();
        assert!(!db.is_empty().await.unwrap());

        // Insert some data
        insert_test_data(&db).await;

        // Connect to the now non-empty database
        let new_db = UserDB::new(test_config.clone(), get_testdb_spec())
            .await
            .unwrap();
        test_rows_present(&new_db).await;

        // Reset
        db.reset().await.unwrap();
        test_rows_absent(&db).await;

        insert_test_data(&db).await;

        // Select query
        log::info!("test select query");
        let query_res = db
            .select("primary", &["email"], "WHERE \"id\" = $1", &[&1])
            .await
            .unwrap();
        assert_eq!(query_res.len(), 1);
        assert_eq!(query_res[0]["email"], "test1@example.com");

        // Test connection to database while having a different expectation of it.
        // This can happen only if the database was modified by something other than
        // this backend.
        log::info!("test connection to changed");
        let new_db = UserDB::new(test_config, get_testdb_spec_alt())
            .await
            .unwrap();
        // The database is the same but we now think that secondary doesn't exist
        assert!(matches!(
            new_db.get_rows_json("secondary").await.unwrap_err(),
            Error::TableNotPresent(name) if name == "secondary"
        ));
        // Reset with no backup should work
        new_db.reset().await.unwrap();
        // Primary table should be empty
        assert!(db.get_rows_json("primary").await.unwrap().is_empty());
        // Secondary table should be absent
        assert!(!db
            .get_all_table_names()
            .await
            .unwrap()
            .contains(&String::from("secondary")))
    }
}
