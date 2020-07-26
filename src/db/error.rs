/// Database manipulation errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Represents all cases of `tokio_postgres::Error`
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
    /// Represents all cases of `json::Error`
    #[error(transparent)]
    Json(#[from] super::json::Error),
}
