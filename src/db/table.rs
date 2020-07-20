use std::collections::HashMap;

type ColSpec = HashMap<String, String>;

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
    pub name: String,
    pub cols: ColSpec,
}

impl Table {
    /// New table with name and a column specification
    pub fn new(name: &str, cols: ColSpec) -> Self {
        Self {
            name: String::from(name),
            cols,
        }
    }
    /// Returns the create query requiring no parameters
    pub fn construct_create_query(&self) -> String {
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
    pub fn contains(&self, colname: &str, coltype: &str) -> bool {
        match self.cols.get(colname) {
            Some(coltype_present) => compare_coltypes(coltype_present, coltype),
            None => false,
        }
    }
}

/// Table column specification
pub struct TableSpec;

impl TableSpec {
    /// admin table
    pub fn admin() -> ColSpec {
        let mut cols = ColSpec::new();
        cols.insert(String::from("id"), String::from("SERIAL"));
        cols.insert(String::from("email"), String::from("TEXT"));
        cols
    }
}
