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
        r#"{"email": "test1@example.com"}"#,
    ));
    sample_data.push(get_primary_entry_from_json(
        r#"{"email": "test2@example.com"}"#,
    ));
    TableJson::new("primary", sample_data)
}

// Test secondary table
fn get_test_secondary_table() -> TableMeta {
    let mut cols = ColSpec::new();
    cols.push(ColMeta::new("id", "INTEGER", ""));
    cols.push(ColMeta::new("timepoint", "INTEGER", ""));
    TableMeta::new(
        "secondary",
        cols,
        "PRIMARY KEY(\"id\", \"timepoint\"),\
        FOREIGN KEY(\"id\") REFERENCES \"primary\"(\"id\")",
    )
}

// Some data for the secondary table
fn get_secondary_sample_data() -> TableJson {
    let mut sample_data = Vec::new();
    sample_data
        .push(get_primary_entry_from_json(r#"{"id": 1, "timepoint": 1}"#));
    sample_data
        .push(get_primary_entry_from_json(r#"{"id": 1, "timepoint": 2}"#));
    TableJson::new("secondary", sample_data)
}

// Test database specification
fn get_testdb_spec() -> TableSpec {
    let mut test_tables = TableSpec::new();
    test_tables.push(get_test_primary_table());
    test_tables.push(get_test_secondary_table());
    test_tables
}

// A different table specification
fn get_testdb_spec_alt() -> TableSpec {
    let mut table_spec = get_testdb_spec();
    // Add an extra table
    let mut extra_cols = Vec::new();
    extra_cols.push(ColMeta::new("id", "SERIAL", "PRIMARY KEY"));
    table_spec.push(TableMeta::new("extra", extra_cols, ""));
    // Remove a table
    table_spec.retain(|t| t.name != "secondary");
    table_spec
}

// Inserts data into the database
async fn insert_test_data(db: &DB) {
    test_rows_absent(db).await;
    let primary_data = get_primary_sample_data();
    db.insert(&primary_data).await.unwrap();
    let secondary_data = get_secondary_sample_data();
    db.insert(&secondary_data).await.unwrap();
    test_rows_present(db).await;
}

// Whether the sample data is present
async fn test_rows_absent(db: &DB) {
    assert!(db.get_rows_json("primary").await.unwrap().is_empty());
    assert!(db.get_rows_json("secondary").await.unwrap().is_empty());
}

// Whether the sample data is absent
async fn test_rows_present(db: &DB) {
    assert_eq!(
        db.get_rows_json("primary").await.unwrap().len(),
        get_primary_sample_data().rows.len()
    );
    assert_eq!(
        db.get_rows_json("secondary").await.unwrap().len(),
        get_secondary_sample_data().rows.len()
    );
}

// Test database
#[tokio::test]
async fn test_db() {
    let _ = pretty_env_logger::try_init();
    let test_config = get_test_config();
    // Manually created database object - use to control what database we
    // are connecting to
    let db = DB {
        name: String::from("odctest"),
        backup_json_path: std::path::PathBuf::from("backup-json/odctest.json"),
        client: connect(&test_config).await.unwrap(),
        tables: get_testdb_spec(),
        was_empty: false,
    };

    // Make sure there are no tables
    db.drop_all_tables().await.unwrap();
    assert!(db.is_empty().await.unwrap());

    // Connect to empty
    let new_db = DB::new(&test_config, get_testdb_spec()).await.unwrap();
    assert!(new_db.was_empty);
    test_rows_absent(&new_db).await;

    // Recreate the tables
    db.create_all_tables().await.unwrap();
    assert!(!db.is_empty().await.unwrap());

    // Insert some data
    insert_test_data(&db).await;

    // Connect to the now non-empty database
    let new_db = DB::new(&test_config, get_testdb_spec()).await.unwrap();
    assert!(!new_db.was_empty);
    test_rows_present(&new_db).await;

    // Reset with backup
    db.reset(true).await.unwrap();
    test_rows_present(&db).await;

    // Reset with no backup
    db.reset(false).await.unwrap();
    test_rows_absent(&db).await;

    insert_test_data(&db).await;

    // Select query
    log::info!("test select query");
    let query_res = db
        .select("primary", &["email"], "WHERE \"id\" = $1", &[&1])
        .await
        .unwrap();
    assert_eq!(query_res.len(), 1);
    assert_eq!(query_res[0]["email"], "test1@example.com");

    // Test connection to database while having a different expectation of it.
    // This can happen only if the database was modified by something other than
    // this backend.
    log::info!("test connection to changed");
    let new_db = DB::new(&test_config, get_testdb_spec_alt()).await.unwrap();
    // The database is the same but we now think that secondary doesn't exist
    assert!(matches!(
        new_db.get_rows_json("secondary").await.unwrap_err(),
        Error::TableNotPresent(name) if name == "secondary"
    ));
    // Reset with backup should fail because the extra table isn't present
    let reset_with_backup = new_db.reset(true).await.unwrap_err();
    assert!(matches!(reset_with_backup, Error::TokioPostgres(_)));
    if let Error::TokioPostgres(err) = reset_with_backup {
        let cause = err.into_source().unwrap();
        assert_eq!(
            cause.to_string(),
            "ERROR: relation \"extra\" does not exist"
        );
    }
    // Reset with no backup should work
    new_db.reset(false).await.unwrap();
    // Primary table should be empty
    assert!(db.get_rows_json("primary").await.unwrap().is_empty());
    // Secondary table should be absent
    assert!(!db
        .get_all_table_names()
        .await
        .unwrap()
        .contains(&String::from("secondary")))
}
