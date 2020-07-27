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
            return format!("{} ({});", init_query, all_columns);
        }
        format!("{} ({}, {});", init_query, all_columns, self.constraints)
    }
    /// Select query
    pub fn construct_select_query(&self, cols: &[String]) -> Result<String> {
        // No specific columns - wildcard
        if cols.is_empty() {
            return Ok(format!("SELECT * FROM \"{}\";", self.name));
        }
        // Check that all requested are present
        let all_cols = self
            .cols
            .iter()
            .map(|c| c.name.clone())
            .collect::<Vec<String>>();
        let cols_not_present = cols
            .iter()
            .filter(|c| !all_cols.contains(c))
            .cloned()
            .collect::<Vec<String>>();
        if !cols_not_present.is_empty() {
            let e = Error::SelectNotPresent(cols_not_present);
            log::error!("{}", e);
            return Err(e);
        }
        // Join into a comma-separated string
        let cols_string = cols
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<String>>()
            .join(",");
        Ok(format!("SELECT {} FROM \"{}\";", cols_string, self.name))
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
    /// Insert query from json
    pub fn construct_insert_query(&self) -> Result<String> {
        if self.rows.is_empty() {
            return Err(Error::InsertEmptyData);
        }
        // Need to make sure keys and values go in the same order
        let mut keys = Vec::with_capacity(self.rows[0].len());
        // Each entry is a vector of values for that row
        let mut row_entries =
            vec![Vec::with_capacity(self.rows[0].len()); self.rows.len()];
        for key in self.rows[0].keys() {
            keys.push(format!("\"{}\"", key));
            for (i, row) in self.rows.iter().enumerate() {
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
            "INSERT INTO \"{}\" ({}) VALUES {};",
            self.name,
            keys.join(","),
            row_entries.join(",")
        ))
    }
}

pub mod error {
    /// Table errors
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        /// Occurs when insert query cannot be constructed due to empty data
        #[error("Data to be inserted is empty")]
        InsertEmptyData,
        /// Occurs when insert query cannot be constructed due to empty data
        #[error("Want to select {0:?} but those columns are not present")]
        SelectNotPresent(Vec<String>),
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
            "CREATE TABLE IF NOT EXISTS \"table\" (\"name\" TEXT );"
        );
        cols.push(ColMeta::new("id", "INTEGER", "PRIMARY KEY"));
        let table = TableMeta::new("table", cols, "");
        assert_eq!(
            table.construct_create_query(),
            "CREATE TABLE IF NOT EXISTS \"table\" (\"name\" TEXT ,\
            \"id\" INTEGER PRIMARY KEY);"
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
            "SELECT * FROM \"table\";"
        );
        assert_eq!(
            table
                .construct_select_query(&[
                    String::from("name"),
                    String::from("id")
                ])
                .unwrap(),
            "SELECT \"name\",\"id\" FROM \"table\";"
        );
        let err = table
            .construct_select_query(&[
                String::from("extra"),
                String::from("id"),
            ])
            .unwrap_err();
        assert!(matches!(
            err,
            Error::SelectNotPresent(cont) if cont == vec![String::from("extra")]
        ));
    }
}
