use crate::{Error, Result};

/// Column specification
pub type ColSpec = Vec<ColMeta>;
/// Table specification
pub type TableSpec = Vec<TableMeta>;
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
        // Ignore primary key because inlining multiple primary keys does not
        // work
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
        // Inlining multiple primary keys doesn't work, so here we are
        let primary_keys = self
            .cols
            .iter()
            .filter(|c| c.primary_key)
            .map(|c| format!("\"{}\"", c.name))
            .collect::<Vec<String>>()
            .join(",");
        let mut primary_key_entry = "".to_string();
        if !primary_keys.is_empty() {
            primary_key_entry = format!(",PRIMARY KEY({})", primary_keys);
        }
        format!(
            "CREATE TABLE \"{}\"({}{})",
            self.name, all_columns, primary_key_entry
        )
    }
    /// Insert query with parameters
    pub fn construct_param_insert_query<T: AsRef<str>>(
        &self,
        cols: &[T],
    ) -> Result<String> {
        self.verify_cols_present(cols)?;

        // The keys and values that will go into the query
        let mut key_entry = Vec::with_capacity(cols.len());
        let mut value_entry = Vec::with_capacity(cols.len());
        for (i, key) in cols.iter().enumerate() {
            key_entry.push(format!("\"{}\"", key.as_ref()));
            value_entry.push(format!("${}", i + 1));
        }

        // Complete query
        Ok(format!(
            "INSERT INTO \"{}\"({}) VALUES({})",
            self.name,
            key_entry.join(","),
            value_entry.join(",")
        ))
    }
    // Checks that a column is present
    fn contains_col<T: AsRef<str>>(&self, colname: T) -> bool {
        for col in &self.cols {
            if col.name == colname.as_ref() {
                return true;
            }
        }
        false
    }
    // Find all columns that are not present
    fn find_cols_not_present<T: AsRef<str>>(&self, cols: &[T]) -> Vec<String> {
        cols.iter()
            .filter(|c| !self.contains_col(c))
            .map(|c| c.as_ref().to_string())
            .collect()
    }
    // Verifies that all the given columns are present
    fn verify_cols_present<T: AsRef<str>>(&self, cols: &[T]) -> Result<()> {
        let cols_not_present = self.find_cols_not_present(cols);
        if !cols_not_present.is_empty() {
            return Err(Error::NoSuchColumns(cols_not_present));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_col() {
        let _ = pretty_env_logger::try_init();
        {
            let col = ColMeta::new().name("name").postgres_type("TEXT");
            assert_eq!(col.construct_create_query_entry(), "\"name\" TEXT");
        }
        {
            let col = ColMeta::new()
                .name("name")
                .postgres_type("TEXT")
                .primary_key(true)
                .unique(true);
            assert_eq!(
                col.construct_create_query_entry(),
                "\"name\" TEXT UNIQUE"
            )
        }
        {
            let col = ColMeta::new()
                .name("name")
                .postgres_type("TEXT")
                .foreign_key(ForeignKey::new("table", "column"));
            assert_eq!(
                col.construct_create_query_entry(),
                "\"name\" TEXT REFERENCES \"table\"(\"column\")"
            )
        }
    }
    #[test]
    fn create_table() {
        let _ = pretty_env_logger::try_init();
        let mut cols = ColSpec::new();

        log::info!("empty columns");
        {
            let table = TableMeta::new("table", cols.clone());
            assert_eq!(
                table.construct_create_query(),
                "CREATE TABLE \"table\"()"
            );
        }

        cols.push(ColMeta::new().name("name").postgres_type("TEXT"));

        log::info!("no primary key");
        {
            let table = TableMeta::new("table", cols.clone());
            assert_eq!(
                table.construct_create_query(),
                "CREATE TABLE \"table\"(\
                    \"name\" TEXT\
                )"
            );
        }

        cols.push(
            ColMeta::new()
                .name("id")
                .postgres_type("INTEGER")
                .primary_key(true),
        );

        log::info!("two columns");
        {
            let table = TableMeta::new("table", cols.clone());
            assert_eq!(
                table.construct_create_query(),
                "CREATE TABLE \"table\"(\
                    \"name\" TEXT,\
                    \"id\" INTEGER,\
                    PRIMARY KEY(\"id\")\
                )"
            );
        }

        cols.push(
            ColMeta::new()
                .name("foreign_id")
                .postgres_type("INTEGER")
                .foreign_key(ForeignKey::new("foreign_table", "foreign_column"))
                .not_null(true)
                .unique(true)
                .primary_key(true),
        );

        log::info!("foreign key");
        {
            let table = TableMeta::new("table", cols);
            assert_eq!(
                table.construct_create_query(),
                "CREATE TABLE \"table\"(\
                    \"name\" TEXT,\
                    \"id\" INTEGER,\
                    \"foreign_id\" INTEGER NOT NULL UNIQUE REFERENCES \
                    \"foreign_table\"(\"foreign_column\"),\
                    PRIMARY KEY(\"id\",\"foreign_id\")\
                )"
            );
        }
    }
    #[test]
    fn insert_table() {
        let _ = pretty_env_logger::try_init();
        let table = crate::tests::get_test_primary_table();
        let table_data = crate::tests::get_primary_data();
        let mut col_names: Vec<String> =
            table_data[0].keys().map(|k| k.to_string()).collect();
        assert_eq!(
            table.construct_param_insert_query(&col_names).unwrap(),
            "INSERT INTO \"primary\"(\"id\",\"email\") VALUES($1,$2)"
        );
        col_names.push("another-name".to_string());
        assert!(matches!(
            table.construct_param_insert_query(&col_names).unwrap_err(),
            Error::NoSuchColumns(cs) if cs == vec!["another-name".to_string()]
        ));
    }
    #[test]
    fn compare_metadata() {
        let primary_meta1 = crate::tests::get_test_primary_table();
        let secondary_meta1 = crate::tests::get_test_secondary_table();
        assert_ne!(primary_meta1, secondary_meta1);

        let mut primary_meta2 = primary_meta1.clone();
        primary_meta2.cols[0].postgres_type = "integer".to_string();
        assert_eq!(primary_meta1, primary_meta2);

        primary_meta2.cols[0].unique = true;
        assert_eq!(primary_meta1, primary_meta2);

        primary_meta2.cols[0].not_null = true;
        assert_eq!(primary_meta1, primary_meta2);

        primary_meta2.cols[0].primary_key = false;
        assert_ne!(primary_meta1, primary_meta2);
    }
}
