/// Column specification
pub type ColSpec = Vec<Column>;

/// Column
pub struct Column {
    pub name: String,
    pub postgres_type: String,
    pub attr: String,
}

impl Column {
    /// New column
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
pub type TableSpec = Vec<Table>;

/// Table
pub struct Table {
    pub name: String,
    pub cols: ColSpec,
    pub constraints: String,
}

impl Table {
    /// New table
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
    pub fn construct_insert_query_json(
        &self,
        row: &serde_json::Map<String, serde_json::Value>,
    ) -> String {
        let mut keys = Vec::<String>::with_capacity(row.len());
        let mut values = Vec::<String>::with_capacity(row.len());
        for (key, value) in row {
            keys.push(format!("\"{}\"", key));
            values.push(format_value_json(value));
        }
        format!(
            "INSERT INTO {} ({}) VALUES ({});",
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

/// Table json
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct TableJson {
    pub name: String,
    pub json: serde_json::Value,
}

impl TableJson {
    pub fn new(name: &str, json: serde_json::Value) -> Self {
        Self {
            name: String::from(name),
            json,
        }
    }
}
