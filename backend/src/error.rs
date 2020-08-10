/// API Errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Represents all cases of `tokio_postgres::Error`
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),
    /// Represents all cases of `argon2::Error`
    #[error(transparent)]
    Argon2(#[from] argon2::Error),
    /// Occurs when insert query cannot be constructed due to empty data
    #[error("want to address table {0} but it does not exist")]
    TableNotPresent(String),
    /// Occurs when a row cannot be parsed as map
    #[error("failed to parse as map: {0}")]
    RowParse(serde_json::Value),
    /// Occurs when insert query cannot be constructed due to empty data
    #[error("data to be inserted is empty")]
    InsertEmptyData,
    /// Occurs when addressing non-existent columns
    #[error("want to address columns {0:?} but they do not exist")]
    ColsNotPresent(Vec<String>),
    /// Pool error
    #[error("error getting connection from DB pool")]
    DBPool(#[from] mobc::Error<tokio_postgres::Error>),
    /// User not found
    #[error("no such user: {0}")]
    NoSuchUser(String),
    /// Token not found
    #[error("no such token: {0}")]
    NoSuchToken(String),
    /// Wrong password
    #[error("wrong password")]
    WrongPassword,
    /// Token too old
    #[error("token too old")]
    TokenTooOld,
    /// Insufficient access
    #[error("Insufficient access")]
    InsufficientAccess,
    /// Represents all cases of `std::io::Error`
    #[error(transparent)]
    IO(#[from] std::io::Error),
    /// Represents all cases of `serde_json::Error`
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    /// Unimplemented value for insert format
    #[error("unimplemented value for insert format: {0}")]
    InsertFormatUnimplemented(serde_json::Value),
    // Wrong authentication type
    #[error("got auth type: {0}; while expected 'basic'")]
    WrongAuthType(String),
    /// All cases of base64 decode error
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),
    /// All cases of utf8 error
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    /// All cases of parse int error
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),
    /// Unexpected access string
    #[error("unexpected access string: {0}")]
    UnexpectedAccessString(String),
    /// Strum error
    #[error(transparent)]
    Strum(#[from] strum::ParseError),
}

impl warp::reject::Reject for Error {}
