use crate::{json, Error, Result};

/// Column specification
pub type ColSpec = Vec<ColMeta>;
/// Table specification
pub type TableSpec = Vec<TableMeta>;
/// Collection of tables in json format
/// Order matters for table creation/data insertion
pub type DBJson = Vec<TableJson>;
/// Row json
pub type RowJson = serde_json::Map<String, serde_json::Value>;

/// Foreign key (column-level)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ForeignKey {
    pub table: String,
    pub column: String,
}

impl ForeignKey {
    pub fn new(table: &str, column: &str) -> Self {
        Self {
            table: table.to_string(),
            column: column.to_string(),
        }
    }
    /// Entry for column-level create query
    pub fn create_query_entry(&self) -> String {
        format!("REFERENCES \"{}\"(\"{}\")", self.table, self.column)
    }
}

/// Column metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColMeta {
    /// Column name
    pub name: String,
    /// Column type as understood by Postgres
    pub postgres_type: String,
    /// Whether it's allowed to be null
    pub not_null: bool,
    /// Whether values are allowed to duplicate
    pub unique: bool,
    /// Whether it's a primary key
    pub primary_key: bool,
    /// Optional foreign key
    pub foreign_key: Option<ForeignKey>,
}

impl ColMeta {
    pub fn name(mut self, val: &str) -> Self {
        self.name = val.to_string();
        self
    }
    pub fn postgres_type(mut self, val: &str) -> Self {
        self.postgres_type = val.to_string();
        self
    }
    pub fn not_null(mut self, val: bool) -> Self {
        self.not_null = val;
        self
    }
    pub fn unique(mut self, val: bool) -> Self {
        self.unique = val;
        self
    }
    pub fn primary_key(mut self, val: bool) -> Self {
        self.primary_key = val;
        self
    }
    pub fn foreign_key(mut self, val: ForeignKey) -> Self {
        self.foreign_key = Some(val);
        self
    }
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            postgres_type: "".to_string(),
            not_null: false,
            unique: false,
            primary_key: false,
            foreign_key: None,
        }
    }
    /// Entry for the create query
    pub fn construct_create_query_entry(&self) -> String {
        let mut entry = format!("\"{}\" {}", self.name, self.postgres_type);
        if self.not_null {
            entry = format!("{} NOT NULL", entry);
        }
        if self.unique {
            entry = format!("{} UNIQUE", entry);
        }
        if self.primary_key {
            entry = format!("{} PRIMARY KEY", entry);
        }
        if let Some(foreign_key) = &self.foreign_key {
            entry = format!("{} {}", entry, foreign_key.create_query_entry());
        }
        entry
    }
}

impl Default for ColMeta {
    fn default() -> Self {
        ColMeta::new()
    }
}

impl PartialEq for ColMeta {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name
            || self.postgres_type.to_lowercase()
                != other.postgres_type.to_lowercase()
            || self.primary_key != other.primary_key
            || self.foreign_key != other.foreign_key
        {
            return false;
        }
        // Don't check unique and not null if primary key
        if self.primary_key {
            return true;
        }
        self.unique == other.unique && self.not_null == other.not_null
    }
}

/// Table metadata
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TableMeta {
    /// Table name
    pub name: String,
    /// Table columns
    pub cols: ColSpec,
}

impl TableMeta {
    pub fn new(name: &str, cols: ColSpec) -> Self {
        Self {
            name: String::from(name),
            cols,
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
        format!("{} ({})", init_query, all_columns)
    }
    /// Select query
    pub fn construct_select_query(
        &self,
        cols: &[&str],
        custom_post: &str,
    ) -> Result<String> {
        // No specific columns - wildcard
        if cols.is_empty() {
            return Ok(format!(
                "SELECT * FROM \"{}\" {}",
                self.name, custom_post
            ));
        }
        // Check that all requested are present
        self.verify_cols_present(cols)?;
        // Join into a comma-separated string
        let cols_string = cols
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<String>>()
            .join(",");
        Ok(format!(
            "SELECT {} FROM \"{}\" {}",
            cols_string, self.name, custom_post
        ))
    }
    /// Select query with a json map per row
    pub fn construct_select_json_query(
        &self,
        cols: &[&str],
        custom_post: &str,
    ) -> Result<String> {
        if cols.is_empty() && custom_post.is_empty() {
            return Ok(format!(
                "SELECT ROW_TO_JSON(\"{0}\") FROM \"{0}\"",
                self.name
            ));
        }
        let inner_query = self.construct_select_query(cols, custom_post)?;
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
            return Err(Error::ColsNotPresent(cols_not_present));
        }
        Ok(())
    }
}

/// Table json
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TableJson {
    /// Table metadata
    pub meta: TableMeta,
    /// Table rows
    pub rows: Vec<RowJson>,
}

impl TableJson {
    pub fn new(meta: TableMeta, rows: Vec<RowJson>) -> Self {
        Self { meta, rows }
    }
}

/// Drop query
pub fn construct_drop_query(name: &str) -> String {
    format!("DROP TABLE IF EXISTS \"{}\" CASCADE", name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_col() {
        let _ = pretty_env_logger::try_init();
        let col = ColMeta::new().name("name").postgres_type("TEXT");
        assert_eq!(col.construct_create_query_entry(), "\"name\" TEXT");
        let col = ColMeta::new()
            .name("name")
            .postgres_type("TEXT")
            .primary_key(true);
        assert_eq!(
            col.construct_create_query_entry(),
            "\"name\" TEXT PRIMARY KEY"
        )
    }
    #[test]
    fn create_table() {
        let _ = pretty_env_logger::try_init();
        let mut cols = ColSpec::new();
        cols.push(ColMeta::new().name("name").postgres_type("TEXT"));
        let table = TableMeta::new("table", cols.clone());
        assert_eq!(
            table.construct_create_query(),
            "CREATE TABLE IF NOT EXISTS \"table\" (\"name\" TEXT)"
        );
        cols.push(
            ColMeta::new()
                .name("id")
                .postgres_type("INTEGER")
                .primary_key(true),
        );
        let table = TableMeta::new("table", cols.clone());
        assert_eq!(
            table.construct_create_query(),
            "CREATE TABLE IF NOT EXISTS \"table\" (\"name\" TEXT,\
            \"id\" INTEGER PRIMARY KEY)"
        );
        cols.push(
            ColMeta::new()
                .name("foreign_id")
                .postgres_type("INTEGER")
                .foreign_key(ForeignKey::new("foreign_table", "foreign_column"))
                .not_null(true)
                .unique(true),
        );
        let table = TableMeta::new("table", cols);
        assert_eq!(
            table.construct_create_query(),
            "CREATE TABLE IF NOT EXISTS \"table\" (\"name\" TEXT,\
            \"id\" INTEGER PRIMARY KEY,\
            \"foreign_id\" INTEGER NOT NULL UNIQUE REFERENCES \
            \"foreign_table\"(\"foreign_column\"))"
        );
    }
    #[test]
    fn select_table() {
        let _ = pretty_env_logger::try_init();
        let mut cols = Vec::new();
        cols.push(ColMeta::new().name("name").postgres_type("TEXT"));
        cols.push(
            ColMeta::new()
                .name("id")
                .postgres_type("INTEGER")
                .primary_key(true),
        );
        let table = TableMeta::new("table", cols);
        assert_eq!(
            table.construct_select_query(&[], "").unwrap(),
            "SELECT * FROM \"table\" "
        );
        assert_eq!(
            table.construct_select_json_query(&[], "").unwrap(),
            "SELECT ROW_TO_JSON(\"table\") FROM \"table\""
        );
        assert_eq!(
            table.construct_select_query(&["name", "id"], "").unwrap(),
            "SELECT \"name\",\"id\" FROM \"table\" "
        );
        assert_eq!(
            table
                .construct_select_json_query(&["name", "id"], "")
                .unwrap(),
            "SELECT ROW_TO_JSON(\"table\") FROM \
            (SELECT \"name\",\"id\" FROM \"table\" ) AS \"table\""
        );
        assert_eq!(
            table
                .construct_select_json_query(
                    &["name", "id"],
                    "WHERE \"name\" = $1"
                )
                .unwrap(),
            "SELECT ROW_TO_JSON(\"table\") FROM \
            (SELECT \"name\",\"id\" FROM \"table\" WHERE \"name\" = $1) \
            AS \"table\""
        );
        let err = table
            .construct_select_query(&["extra", "id"], "")
            .unwrap_err();
        assert!(matches!(
            err,
            Error::ColsNotPresent(cont) if cont == vec![String::from("extra")]
        ));
    }
    #[test]
    fn insert_table() {
        let _ = pretty_env_logger::try_init();
        let mut cols = Vec::new();
        cols.push(ColMeta::new().name("name").postgres_type("TEXT"));
        cols.push(
            ColMeta::new()
                .name("id")
                .postgres_type("INTEGER")
                .primary_key(true),
        );
        let table = TableMeta::new("table", cols);
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
            "INSERT INTO \"table\" (\"name\",\"id\") VALUES \
            ('alice','1'),('bob','2')"
        );
    }
    #[test]
    fn compare_metadata() {
        let primary_meta1 = crate::tests::get_test_primary_table();
        let secondary_meta1 = crate::tests::get_test_secondary_table();
        assert_eq!(primary_meta1, primary_meta1);
        assert_eq!(secondary_meta1, secondary_meta1);
        assert_ne!(primary_meta1, secondary_meta1);

        let mut primary_meta2 = primary_meta1.clone();
        primary_meta2.cols[0].postgres_type = "integer".to_string();
        assert_eq!(primary_meta1, primary_meta2);

        primary_meta2.cols[0].unique = true;
        assert_eq!(primary_meta1, primary_meta2);

        primary_meta2.cols[0].not_null = true;
        assert_eq!(primary_meta1, primary_meta2);
    }
}
