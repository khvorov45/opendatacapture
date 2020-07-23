/// Column specification
pub type ColSpec = Vec<ColMeta>;

/// Column metadata
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

/// Table specification
pub type TableSpec = Vec<TableMeta>;

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
    /// Insert query from json
    pub fn construct_insert_query_json(&self, row: &RowJson) -> String {
        let mut keys = Vec::<String>::with_capacity(row.len());
        let mut values = Vec::<String>::with_capacity(row.len());
        for (key, value) in row {
            keys.push(format!("\"{}\"", key));
            values.push(format_value_json(value));
        }
        format!(
            "INSERT INTO \"{}\" ({}) VALUES ({});",
            self.name,
            keys.join(","),
            values.join(",")
        )
    }
}

/// Formats a json value for insert
pub fn format_value_json(value: &serde_json::Value) -> String {
    use serde_json::Value;
    match value {
        Value::String(v) => format!("\'{}\'", v),
        Value::Number(n) => format!("\'{}\'", n),
        _ => {
            log::error!("unimplemented value type: {:?}", value);
            String::from("")
        }
    }
}

/// Collection of tables
/// Order matters for creation/data insertion
pub type DBJson = Vec<TableJson>;

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

/// Row json
pub type RowJson = serde_json::Map<String, serde_json::Value>;
