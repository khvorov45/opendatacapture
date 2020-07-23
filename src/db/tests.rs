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

// Test primary table
fn get_test_primary_table() -> TableMeta {
    let mut cols = ColSpec::new();
    cols.push(ColMeta::new("id", "SERIAL", "PRIMARY KEY"));
    cols.push(ColMeta::new("email", "TEXT", "NOT NULL"));
    TableMeta::new("primary", cols, "")
}

// One entry for the primary table
fn get_primary_entry_from_json(json: &str) -> RowJson {
    serde_json::from_str::<RowJson>(json).unwrap()
}

// Some data for the primary table
fn get_primary_sample_data() -> TableJson {
    let mut sample_data = Vec::new();
    sample_data.push(get_primary_entry_from_json(
        r#"{"email": "test@example.com"}"#,
    ));
    TableJson::new("primary", sample_data)
}

// Test secondary table
fn get_test_secondary_table() -> TableMeta {
    let mut cols = ColSpec::new();
    cols.push(ColMeta::new("id", "INTEGER", "PRIMARY KEY"));
    cols.push(ColMeta::new("timepoint", "INTEGER", "NOT NULL"));
    TableMeta::new(
        "secondary",
        cols,
        "FOREIGN KEY(\"id\") REFERENCES \"primary\"(\"id\")",
    )
}

// Test database specification
fn get_testdb_spec() -> TableSpec {
    let mut test_tables = TableSpec::new();
    test_tables.push(get_test_primary_table());
    test_tables.push(get_test_secondary_table());
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
    assert!(db.get_rows_json("primary").await.unwrap().is_empty());
    assert!(db.get_rows_json("secondary").await.unwrap().is_empty());
    // Insert some data
    let primary_data = get_primary_sample_data();
    db.insert(&primary_data).await.unwrap();
    assert_eq!(
        db.get_rows_json("primary").await.unwrap().len(),
        primary_data.rows.len()
    );
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
    assert_eq!(db.get_rows_json("primary").await.unwrap().len(), 1)
}

// No backup
async fn test_connection_no_backup() {
    let db = get_testdb(false).await;
    db.init(false).await.unwrap();
    // The one test row should not be preserved
    assert!(db.get_rows_json("primary").await.unwrap().is_empty());
}

#[tokio::test]
async fn test_db() {
    pretty_env_logger::init();
    log::info!("test connection to empty");
    test_connection_to_empty().await;
    log::info!("test connection to non-empty");
    test_connection_to_nonempty().await;
}
