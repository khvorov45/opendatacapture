/// API Errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Admin database errors
    #[error(transparent)]
    AdminDB(#[from] super::admindb::Error),
}
