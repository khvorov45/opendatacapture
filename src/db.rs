use log::{debug, error};
use std::collections::HashMap;
use tokio_postgres::{Client, Error};

use crate::error::APIError;

/// Administrative database
pub struct DB {
    client: Client,
    tables: Vec<Table>,
}

impl DB {
    /// Create a new database given a reference to config.
    /// Checks that the structure is correct before returning.
    /// Checks that the tables are correct before returning.
    pub async fn new(
        config: &tokio_postgres::Config,
        tables: Vec<Table>,
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
        let expected_tables: Vec<&String> =
            self.tables.iter().map(|t| &t.name).collect();
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
        for table in &self.tables {
            self.client
                .execute(table.construct_create_query().as_str(), &[])
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
        for table in &self.tables {
            // Pull all names and types
            let names_and_types = self
                .client
                .query(
                    "SELECT column_name, data_type \
                FROM information_schema.columns \
                WHERE table_name = $1",
                    &[&table.name],
                )
                .await?;
            // Compare to the expected names and types
            let mut table_is_wrong = false;
            for row in &names_and_types {
                let colname: String = row.get(0);
                let coltype: String = row.get(1);
                if !table.contains(&colname, &coltype) {
                    debug!(
                        "Table \"{}\" should not contain column \"{}\" \
                        with type \"{}\"",
                        table.name, colname, coltype
                    );
                    table_is_wrong = true;
                    break;
                }
            }
            if table.cols.len() != names_and_types.len() {
                debug!(
                    "Table \"{}\" has {} column(s) while expected {}",
                    &table.name,
                    names_and_types.len(),
                    table.cols.len(),
                );
                table_is_wrong = true;
            }
            if table_is_wrong {
                incorrect_tables.push(String::from(&table.name));
            }
        }
        debug!("Found incorrect tables: {:?}", incorrect_tables);
        Ok(incorrect_tables)
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

/// Compares 2 column types. Case-insensitive.
fn compare_coltypes(t1: &str, t2: &str) -> bool {
    recode_coltype(t1) == recode_coltype(t2)
}

/// Recodes coltypes to make them comparable
fn recode_coltype(coltype: &str) -> String {
    let coltype = coltype.to_lowercase();
    let mut codes = HashMap::new();
    codes.insert("serial", "integer");
    match codes.get(coltype.as_str()) {
        Some(alt) => String::from(*alt),
        None => coltype,
    }
}

/// A standard table
pub struct Table {
    name: String,
    cols: HashMap<String, String>,
}

impl Table {
    /// New table with name and a column specification
    fn new(name: &str, cols: HashMap<String, String>) -> Self {
        Self {
            name: String::from(name),
            cols,
        }
    }
    /// Returns the create query requiring no parameters
    fn construct_create_query(&self) -> String {
        let coltypes = self
            .cols
            // Surround colnames by quotation marks
            .iter()
            .map(|(colname, coltype)| format! {"\"{}\" {}", colname, coltype})
            .collect::<Vec<String>>()
            // Join into a comma-separated string
            .join(",");
        format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" ({});",
            self.name, coltypes
        )
    }
    /// Checks that the table has the given column with the given type
    fn contains(&self, colname: &str, coltype: &str) -> bool {
        match self.cols.get(colname) {
            Some(coltype_present) => compare_coltypes(coltype_present, coltype),
            None => false,
        }
    }
}

/// Table column specification
struct TableSpec;

impl TableSpec {
    /// admin table
    fn admin() -> HashMap<String, String> {
        let mut cols = HashMap::new();
        cols.insert(String::from("id"), String::from("SERIAL"));
        cols.insert(String::from("email"), String::from("TEXT"));
        cols
    }
}

/// Table set for one database
pub struct Tableset;

impl Tableset {
    /// The administrative database
    pub fn admin() -> Vec<Table> {
        vec![Table::new("admin", TableSpec::admin())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Assume that there is a database called odctest,
    // connect with the same user and password
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

    // Clear (no tables) test database
    async fn get_clear_test_db() -> DB {
        let db = DB {
            client: connect(&get_test_config()).await.unwrap(),
            tables: vec![Table::new("admin", TableSpec::admin())],
        };
        db.clear().await.unwrap();
        db
    }

    // Check that the given database is empty
    async fn is_empty(db: &DB) -> bool {
        let all_tables = db.get_all_table_names().await.unwrap();
        all_tables.is_empty() && (db.state().await.unwrap() == DBState::Empty)
    }

    // Inserts a table
    async fn insert_table(db: &DB, table: &str) {
        db.client
            .execute(format!("CREATE TABLE {};", table).as_str(), &[])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_db() {
        pretty_env_logger::init();
        // Connect to empty test database
        let db = get_clear_test_db().await;
        // Check that it's empty
        assert!(is_empty(&db).await);
        // Initialise
        db.init().await.unwrap();
        // Not empty
        assert!(!is_empty(&db).await);
        // Should now be correct
        assert_eq!(db.state().await.unwrap(), DBState::Correct);
        // Should remain correct after reset
        db.reset().await.unwrap();
        assert_eq!(db.state().await.unwrap(), DBState::Correct);
        // Insert an extra table
        insert_table(&db, "extratable (name TEXT)").await;
        // See if the incorrect state is detected
        assert_eq!(db.state().await.unwrap(), DBState::Incorrect);
        // Reset
        db.reset().await.unwrap();
        // Now should be correct
        assert_eq!(db.state().await.unwrap(), DBState::Correct);
        // Clear
        db.clear().await.unwrap();
        // Check that it's empty
        assert!(is_empty(&db).await);
        // Create a table that looks like what we want but has
        // a wrong type
        insert_table(&db, "admin (id SERIAL, name CHAR(50))").await;
        // Table admin is incorrect
        assert_eq!(db.find_incorrect_tables().await.unwrap(), ["admin"]);
        // Clear and make correct
        db.clear().await.unwrap();
        db.init().await.unwrap();
        assert_eq!(db.state().await.unwrap(), DBState::Correct);
        assert!(db.find_incorrect_tables().await.unwrap().is_empty());
        // Add an extra column to the table
        db.client
            .execute("ALTER TABLE admin ADD extravar TEXT;", &[])
            .await
            .unwrap();
        // Now admin is incorrect
        assert_eq!(db.find_incorrect_tables().await.unwrap(), ["admin"]);
    }
}
