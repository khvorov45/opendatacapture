/// Database manipulation errors
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Represents all cases of `tokio_postgres::Error`
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
    /// Represents all cases of `json::Error`
    #[error(transparent)]
    Json(#[from] super::json::Error),
    /// Represents all cases of `db::table::Error`
    #[error(transparent)]
    Table(#[from] super::table::Error),
    /// Occurs when insert query cannot be constructed due to empty data
    #[error("want to address table {0} but it does not exist")]
    TableNotPresent(String),
}
