/// API Errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    // My errors --------------------------------------------------------------
    /// Occurs when insert query cannot be constructed due to empty data
    #[error("want to address table {0} but it does not exist")]
    NoSuchTable(String),

    /// Create a table that is already present
    #[error("table \"{0}\" already exists")]
    TableAlreadyExists(String),

    /// Occurs when a row cannot be parsed as map
    #[error("failed to parse as map: {0}")]
    RowParse(serde_json::Value),

    /// Occurs when insert query cannot be constructed due to empty data
    #[error("data to be inserted is empty")]
    InsertEmptyData,

    /// Occurs when addressing non-existent columns
    #[error("want to address columns {0:?} but they do not exist")]
    NoSuchColumns(Vec<String>),

    /// Unimplemented value for insert format
    #[error("unimplemented value for insert format: {0}")]
    InsertFormatUnimplemented(serde_json::Value),

    /// Unexpected access string
    #[error("unexpected access string: {0}")]
    UnexpectedAccessString(String),

    /// Unauthorized
    #[error(transparent)]
    Unauthorized(#[from] Unauthorized),

    /// User ID not found
    #[error("no such user id: {0}")]
    NoSuchUserId(i32),

    /// User email not found
    #[error("no such user email: {0}")]
    NoSuchUserEmail(String),

    /// Project not found
    #[error("no such project: {1} for user id: {0}")]
    NoSuchProject(i32, String),

    /// Project already exists
    #[error("project: {1} already exists for user id: {0}")]
    ProjectAlreadyExists(i32, String),

    /// Database name not found
    #[error("no such database: {0}")]
    NoSuchDatabase(String),

    // Not my errors ----------------------------------------------------------
    /// Represents all cases of `sqlx::Error`
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    /// Represents all cases of `argon2::Error`
    #[error(transparent)]
    Argon2(#[from] argon2::Error),

    /// Represents all cases of `std::io::Error`
    #[error(transparent)]
    IO(#[from] std::io::Error),

    /// Represents all cases of `serde_json::Error`
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// All cases of base64 decode error
    #[error(transparent)]
    Base64(#[from] base64::DecodeError),

    /// All cases of utf8 error
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    /// All cases of parse int error
    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    /// All cases of chrono error
    #[error(transparent)]
    Chrono(#[from] chrono::ParseError),
}

impl warp::reject::Reject for Error {}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Unauthorized {
    /// User email not found
    #[error("no such user email: {0}")]
    NoSuchUserEmail(String),

    /// Token not found
    #[error("no such token: {0}")]
    NoSuchToken(String),

    /// Wrong password
    #[error("wrong password: {0}")]
    WrongPassword(String),

    /// Token too old
    #[error("token too old")]
    TokenTooOld,

    /// Insufficient access
    #[error("insufficient access")]
    InsufficientAccess,

    /// Wrong authentication type
    #[error("got auth type: {0}; while expected 'Bearer'")]
    WrongAuthType(String),
}
