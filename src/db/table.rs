use super::json;

pub use error::Error;

/// Result
pub type Result<T> = std::result::Result<T, Error>;
/// Column specification
pub type ColSpec = Vec<ColMeta>;
/// Table specification
pub type TableSpec = Vec<TableMeta>;
/// Collection of tables in json format
/// Order matters for table creation/data insertion
pub type DBJson = Vec<TableJson>;
/// Row json
pub type RowJson = serde_json::Map<String, serde_json::Value>;

/// Column metadata
#[derive(Debug, Clone, PartialEq)]
pub struct ColMeta {
    /// Column name
    pub name: String,
    /// Column type as understood by Postgres
    pub postgres_type: String,
    /// Extra column attributes, e.g., `NOT NULL`
    pub attr: String,
}

impl ColMeta {
    pub fn new(name: &str, postgres_type: &str, attr: &str) -> Self {
        Self {
            name: String::from(name),
            postgres_type: String::from(postgres_type),
            attr: String::from(attr),
        }
    }
    /// Entry for the create query
    pub fn construct_create_query_entry(&self) -> String {
        format!("\"{}\" {} {}", self.name, self.postgres_type, self.attr)
    }
}

/// Table metadata
#[derive(Debug, Clone, PartialEq)]
pub struct TableMeta {
    /// Table name
    pub name: String,
    /// Table columns
    pub cols: ColSpec,
    /// Table constraints, e.g., `FOREIGN KEY`
    pub constraints: String,
}

impl TableMeta {
    pub fn new(name: &str, cols: ColSpec, constraints: &str) -> Self {
        Self {
            name: String::from(name),
            cols,
            constraints: String::from(constraints),
        }
    }
    /// Create query
    pub fn construct_create_query(&self) -> String {
        let all_columns: String = self
            .cols
            .iter()
            .map(|c| c.construct_create_query_entry())
            .collect::<Vec<String>>()
            .join(",");
        let init_query =
            format!("CREATE TABLE IF NOT EXISTS \"{}\"", self.name);
        if self.constraints.is_empty() {
            return format!("{} ({})", init_query, all_columns);
        }
        format!("{} ({}, {})", init_query, all_columns, self.constraints)
    }
    /// Select query
    pub fn construct_select_query(&self, cols: &[&str]) -> Result<String> {
        // No specific columns - wildcard
        if cols.is_empty() {
            return Ok(format!("SELECT * FROM \"{}\"", self.name));
        }
        // Check that all requested are present
        self.verify_cols_present(cols)?;
        // Join into a comma-separated string
        let cols_string = cols
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<String>>()
            .join(",");
        Ok(format!("SELECT {} FROM \"{}\"", cols_string, self.name))
    }
    /// Select query with a json map per row
    pub fn construct_select_json_query(&self, cols: &[&str]) -> Result<String> {
        if cols.is_empty() {
            return Ok(format!(
                "SELECT ROW_TO_JSON(\"{0}\") FROM \"{0}\"",
                self.name
            ));
        }
        let inner_query = self.construct_select_query(cols)?;
        Ok(format!(
            "SELECT ROW_TO_JSON(\"{0}\") FROM ({1}) AS \"{0}\"",
            self.name, inner_query
        ))
    }
    // Insert query
    pub fn construct_insert_query(&self, rows: &[RowJson]) -> Result<String> {
        if rows.is_empty() {
            return Err(Error::InsertEmptyData);
        }
        // Make sure all the columns are present
        let cols = rows[0].keys().map(|k| k.as_str()).collect::<Vec<&str>>();
        self.verify_cols_present(&cols)?;
        // Need to make sure keys and values go in the same order
        let mut keys = Vec::with_capacity(cols.len());
        // Each entry is a vector of values for that row
        let mut row_entries =
            vec![Vec::with_capacity(rows[0].len()); rows.len()];
        for key in cols {
            keys.push(format!("\"{}\"", key));
            for (i, row) in rows.iter().enumerate() {
                row_entries[i].push(json::insert_format(&row[key])?);
            }
        }
        // Format each entry
        let row_entries: Vec<String> = row_entries
            .iter()
            .map(|r| r.join(",")) // comma-separated list
            .map(|r| format!("({})", r)) // surround by paretheses
            .collect();
        Ok(format!(
            "INSERT INTO \"{}\" ({}) VALUES {}",
            self.name,
            keys.join(","),
            row_entries.join(",")
        ))
    }
    // Checks that a column is present
    fn contains_col(&self, colname: &str) -> bool {
        for col in &self.cols {
            if col.name == colname {
                return true;
            }
        }
        false
    }
    // Find all columns that are not present
    fn find_cols_not_present(&self, cols: &[&str]) -> Vec<String> {
        cols.iter()
            .filter(|c| !self.contains_col(c))
            .map(|c| String::from(*c))
            .collect()
    }
    // Verifies that all the given columns are present
    fn verify_cols_present(&self, cols: &[&str]) -> Result<()> {
        let cols_not_present = self.find_cols_not_present(cols);
        if !cols_not_present.is_empty() {
            let e = Error::ColsNotPresent(cols_not_present);
            log::error!("{}", e);
            return Err(e);
        }
        Ok(())
    }
}

/// Table json
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TableJson {
    /// Table name
    pub name: String,
    /// Table rows
    pub rows: Vec<RowJson>,
}

impl TableJson {
    pub fn new(name: &str, rows: Vec<RowJson>) -> Self {
        Self {
            name: String::from(name),
            rows,
        }
    }
}

pub mod error {
    /// Table errors
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        /// Occurs when insert query cannot be constructed due to empty data
        #[error("data to be inserted is empty")]
        InsertEmptyData,
        /// Occurs when addressing non-existent columns
        #[error("want to address columns {0:?} but they do not exist")]
        ColsNotPresent(Vec<String>),
        /// Represents all cases of `json::Error`
        #[error(transparent)]
        Json(#[from] super::json::Error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_col() {
        let col = ColMeta::new("name", "TEXT", "");
        assert_eq!(col.construct_create_query_entry(), "\"name\" TEXT ");
        let col = ColMeta::new("name", "TEXT", "PRIMARY KEY");
        assert_eq!(
            col.construct_create_query_entry(),
            "\"name\" TEXT PRIMARY KEY"
        )
    }
    #[test]
    fn create_table() {
        let mut cols = Vec::new();
        cols.push(ColMeta::new("name", "TEXT", ""));
        let table = TableMeta::new("table", cols.clone(), "");
        assert_eq!(
            table.construct_create_query(),
            "CREATE TABLE IF NOT EXISTS \"table\" (\"name\" TEXT )"
        );
        cols.push(ColMeta::new("id", "INTEGER", "PRIMARY KEY"));
        let table = TableMeta::new("table", cols, "");
        assert_eq!(
            table.construct_create_query(),
            "CREATE TABLE IF NOT EXISTS \"table\" (\"name\" TEXT ,\
            \"id\" INTEGER PRIMARY KEY)"
        );
    }
    #[test]
    fn select_table() {
        let mut cols = Vec::new();
        cols.push(ColMeta::new("name", "TEXT", ""));
        cols.push(ColMeta::new("id", "INTEGER", "PRIMARY KEY"));
        let table = TableMeta::new("table", cols, "");
        assert_eq!(
            table.construct_select_query(&[]).unwrap(),
            "SELECT * FROM \"table\""
        );
        assert_eq!(
            table.construct_select_json_query(&[]).unwrap(),
            "SELECT ROW_TO_JSON(\"table\") FROM \"table\""
        );
        assert_eq!(
            table.construct_select_query(&["name", "id"]).unwrap(),
            "SELECT \"name\",\"id\" FROM \"table\""
        );
        assert_eq!(
            table.construct_select_json_query(&["name", "id"]).unwrap(),
            "SELECT ROW_TO_JSON(\"table\") FROM \
            (SELECT \"name\",\"id\" FROM \"table\") AS \"table\""
        );
        let err = table.construct_select_query(&["extra", "id"]).unwrap_err();
        assert!(matches!(
            err,
            Error::ColsNotPresent(cont) if cont == vec![String::from("extra")]
        ));
    }
    #[test]
    fn insert_table() {
        let mut cols = Vec::new();
        cols.push(ColMeta::new("name", "TEXT", ""));
        cols.push(ColMeta::new("id", "INTEGER", "PRIMARY KEY"));
        let table = TableMeta::new("table", cols, "");
        let mut rows = Vec::new();
        let mut row1 = RowJson::new();
        row1.insert(
            "name".to_string(),
            serde_json::Value::String("alice".to_string()),
        );
        rows.push(row1.clone());
        assert_eq!(
            table.construct_insert_query(&rows).unwrap(),
            "INSERT INTO \"table\" (\"name\") VALUES ('alice')"
        );
        let mut row2 = RowJson::new();
        row2.insert(
            "name".to_string(),
            serde_json::Value::String("bob".to_string()),
        );
        rows.push(row2.clone());
        assert_eq!(
            table.construct_insert_query(&rows).unwrap(),
            "INSERT INTO \"table\" (\"name\") VALUES ('alice'),('bob')"
        );
        row1.insert(
            "id".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(1.0).unwrap(),
            ),
        );
        row2.insert(
            "id".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(2.0).unwrap(),
            ),
        );
        let rows = vec![row1, row2];
        assert_eq!(
            table.construct_insert_query(&rows).unwrap(),
            "INSERT INTO \"table\" (\"id\",\"name\") VALUES \
            ('1','alice'),('2','bob')"
        );
    }
}
