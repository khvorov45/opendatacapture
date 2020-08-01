/// Utilities for json handling
pub use error::Error;

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

pub type Result<T> = std::result::Result<T, Error>;

pub mod error {
    /// JSON manipulation errors
    #[derive(Debug, thiserror::Error)]
    pub enum Error {
        /// Represents all cases of `std::io::Error`
        #[error(transparent)]
        IO(#[from] std::io::Error),
        /// Represents all cases of `serde_json::Error`
        #[error(transparent)]
        SerdeJson(#[from] serde_json::Error),
        /// Unimplemented value for insert format
        #[error("unimplemented value for insert format: {0}")]
        InsertFormatUnimplemented(serde_json::Value),
    }
}
