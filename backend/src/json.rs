/// Utilities for json handling
use crate::{Error, Result};

/// Write json
pub fn write<T: serde::Serialize>(
    json: T,
    filepath: &std::path::Path,
) -> Result<()> {
    let file = std::fs::File::create(filepath)?;
    serde_json::to_writer(&file, &serde_json::to_value(json)?)?;
    Ok(())
}

/// Read json
pub fn read<T: serde::de::DeserializeOwned>(
    filepath: &std::path::Path,
) -> Result<T> {
    let file = std::fs::File::open(filepath)?;
    let reader = std::io::BufReader::new(file);
    let json: T = serde_json::from_reader(reader)?;
    Ok(json)
}

pub fn insert_format(value: &serde_json::Value) -> Result<String> {
    use serde_json::Value;
    match value {
        Value::String(v) => Ok(format!("\'{}\'", v)),
        Value::Number(n) => Ok(format!("\'{}\'", n)),
        other => {
            let e = Error::InsertFormatUnimplemented(other.clone());
            log::error!("{}", e);
            Err(e)
        }
    }
}
