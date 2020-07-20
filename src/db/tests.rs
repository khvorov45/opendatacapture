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
    let db = DB {
        client: connect(&get_test_config()).await.unwrap(),
        tables: vec![Table::new("admin", TableSpec::admin())],
    };
    db.clear().await.unwrap();
    db
}

// Check that the given database is empty
async fn is_empty(db: &DB) -> bool {
    let all_tables = db.get_all_table_names().await.unwrap();
    all_tables.is_empty() && (db.state().await.unwrap() == DBState::Empty)
}

// Inserts a table
async fn insert_table(db: &DB, table: &str) {
    db.client
        .execute(format!("CREATE TABLE {};", table).as_str(), &[])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_db() {
    pretty_env_logger::init();
    // Connect to empty test database
    let db = get_clear_test_db().await;
    // Check that it's empty
    assert!(is_empty(&db).await);
    // Initialise
    db.init().await.unwrap();
    // Not empty
    assert!(!is_empty(&db).await);
    // Should now be correct
    assert_eq!(db.state().await.unwrap(), DBState::Correct);
    // Should remain correct after reset
    db.reset().await.unwrap();
    assert_eq!(db.state().await.unwrap(), DBState::Correct);
    // Insert an extra table
    insert_table(&db, "extratable (name TEXT)").await;
    // See if the incorrect state is detected
    assert_eq!(db.state().await.unwrap(), DBState::Incorrect);
    // Reset
    db.reset().await.unwrap();
    // Now should be correct
    assert_eq!(db.state().await.unwrap(), DBState::Correct);
    // Clear
    db.clear().await.unwrap();
    // Check that it's empty
    assert!(is_empty(&db).await);
    // Create a table that looks like what we want but has
    // a wrong type
    insert_table(&db, "admin (id SERIAL, name CHAR(50))").await;
    // Table admin is incorrect
    assert_eq!(db.find_incorrect_tables().await.unwrap(), ["admin"]);
    // Clear and make correct
    db.clear().await.unwrap();
    db.init().await.unwrap();
    assert_eq!(db.state().await.unwrap(), DBState::Correct);
    assert!(db.find_incorrect_tables().await.unwrap().is_empty());
    // Add an extra column to the table
    db.client
        .execute("ALTER TABLE admin ADD extravar TEXT;", &[])
        .await
        .unwrap();
    // Now admin is incorrect
    assert_eq!(db.find_incorrect_tables().await.unwrap(), ["admin"]);
}
