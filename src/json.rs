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
    }
}
