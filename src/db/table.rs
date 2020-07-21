use log::debug;
use std::collections::HashMap;

pub type ColSpec = HashMap<String, String>;

/// Compares 2 column types. Case-insensitive.
pub fn compare_coltypes(t1: &str, t2: &str) -> bool {
    recode_coltype(t1) == recode_coltype(t2)
}

/// Recodes coltypes to make them comparable
fn recode_coltype(coltype: &str) -> String {
    let coltype = coltype.to_lowercase();
    match super::constants::TYPES.get(coltype.as_str()) {
        Some(alt) => String::from(*alt),
        None => coltype,
    }
}

/// Returns the create query requiring no parameters
pub fn construct_create_query(name: &str, cols: &ColSpec) -> String {
    let coltypes = cols
        // Surround colnames by quotation marks
        .iter()
        .map(|(colname, coltype)| format! {"\"{}\" {}", colname, coltype})
        .collect::<Vec<String>>()
        // Join into a comma-separated string
        .join(",");
    format!("CREATE TABLE IF NOT EXISTS \"{}\" ({});", name, coltypes)
}

pub fn verify(
    name: &str,
    cols_obtained: &ColSpec,
    cols_expected: &ColSpec,
) -> bool {
    // Check number of columns
    if cols_expected.len() != cols_obtained.len() {
        debug!(
            "Table \"{}\" is expected to have {} columns but has {}",
            name,
            cols_expected.len(),
            cols_obtained.len()
        );
        return false;
    }
    // Name or type may be wrong
    for (colname_obtained, coltype_obtained) in cols_obtained {
        if !verify_column(colname_obtained, coltype_obtained, cols_expected) {
            debug!(
                "Table \"{}\" column \"{}\" failed verification",
                name, colname_obtained
            );
            return false;
        }
    }
    true
}

/// Checks if given column name and type are present in ColSpec
fn verify_column(
    colname: &str,
    coltype: &str,
    cols_expected: &ColSpec,
) -> bool {
    match cols_expected.get(colname) {
        // This column name should be in the table
        Some(coltype_expected) => {
            if !compare_coltypes(coltype_expected, coltype) {
                debug!(
                    "Column \"{}\" has type \"{}\" while expected \"{}\"",
                    colname, coltype, coltype_expected
                );
                return false;
            }
            true
        }
        // Obtained column name is not expected
        None => {
            debug!("Column \"{}\" is not expected", colname);
            false
        }
    }
}
