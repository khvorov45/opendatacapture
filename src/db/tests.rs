use super::*;

// Assume that there is a database called odctest,
// connect with the same user and password
fn get_test_config() -> tokio_postgres::Config {
    let mut dbconfig = tokio_postgres::Config::new();
    dbconfig
        .host("localhost")
        .port(5432)
        .dbname("odctest")
        .user("odcapi")
        .password("odcapi");
    dbconfig
}

// Clear (no tables) test database
async fn get_clear_test_db() -> DB {
    let mut test_tables = TableSpec::new();
    let mut admin_cols = ColSpec::new();
    admin_cols.push(Column::new("id", "SERIAL", "PRIMARY KEY"));
    admin_cols.push(Column::new("email", "TEXT", "NOT NULL"));
    test_tables.push(Table::new("admin", admin_cols, ""));
    let db = DB {
        client: connect(&get_test_config()).await.unwrap(),
        tables: test_tables,
    };
    db.drop_all_tables().await.unwrap();
    db
}

// Tests connection to an empty database
async fn test_connection_to_empty() {
    // Connect to empty test database
    let db = get_clear_test_db().await;
    // Check that it's empty
    assert!(db.is_empty().await.unwrap());
    // Initialise
    db.create_all_tables().await.unwrap();
    // Not empty
    assert!(!db.is_empty().await.unwrap());
    // Empty again
    db.drop_all_tables().await.unwrap();
    assert!(db.is_empty().await.unwrap());
    // Init as it would be normally
    db.init(true).await.unwrap();
    assert!(!db.is_empty().await.unwrap());
}

#[tokio::test]
async fn test_db() {
    pretty_env_logger::init();
    test_connection_to_empty().await;
}
