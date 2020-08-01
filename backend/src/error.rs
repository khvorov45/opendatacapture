/// API Errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Admin database errors
    #[error(transparent)]
    DB(#[from] super::db::Error),
}
