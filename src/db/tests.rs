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

// Test database specification
fn get_testdb_spec() -> TableSpec {
    let mut test_tables = TableSpec::new();
    let mut admin_cols = ColSpec::new();
    admin_cols.push(Column::new("id", "SERIAL", "PRIMARY KEY"));
    admin_cols.push(Column::new("email", "TEXT", "NOT NULL"));
    test_tables.push(Table::new("admin", admin_cols, ""));
    test_tables
}

// Test database
async fn get_testdb(clear: bool) -> DB {
    // Manually created database object
    let db = DB {
        backup_json_path: std::path::PathBuf::from("backup-json/test.json"),
        client: connect(&get_test_config()).await.unwrap(),
        tables: get_testdb_spec(),
    };
    // Make sure there are no tables
    db.drop_all_tables().await.unwrap();
    assert!(db.is_empty().await.unwrap());
    if clear {
        return db;
    }
    // Recreate the tables
    db.create_all_tables().await.unwrap();
    assert!(!db.is_empty().await.unwrap());
    // Tables should be empty
    assert!(db.get_rows_json("admin").await.unwrap().is_empty());
    let admin1 = r#"
        {
            "email": "test1@example.com"
        }"#;
    let admin1_json: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(admin1).unwrap();
    db.client
        .execute(
            db.tables[0]
                .construct_insert_query_json(&admin1_json)
                .as_str(),
            &[],
        )
        .await
        .unwrap();
    assert_eq!(db.get_rows_json("admin").await.unwrap().len(), 1);
    db
}

// Tests connection to an empty database
async fn test_connection_to_empty() {
    // Connect to empty test database
    let db = get_testdb(true).await;
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

// Tests connection to a non-empty database
async fn test_connection_to_nonempty() {
    log::info!("test connection with backup");
    test_connection_with_backup().await;
    log::info!("test connection with no backup");
    test_connection_no_backup().await;
}

// Backup
async fn test_connection_with_backup() {
    let db = get_testdb(false).await;
    db.init(true).await.unwrap();
    // The one test row should be preserved
    assert_eq!(db.get_rows_json("admin").await.unwrap().len(), 1)
}

// No backup
async fn test_connection_no_backup() {
    let db = get_testdb(false).await;
    db.init(false).await.unwrap();
    // The one test row should not be preserved
    assert!(db.get_rows_json("admin").await.unwrap().is_empty());
}

#[tokio::test]
async fn test_db() {
    pretty_env_logger::init();
    log::info!("test connection to empty");
    test_connection_to_empty().await;
    log::info!("test connection to non-empty");
    test_connection_to_nonempty().await;
}
