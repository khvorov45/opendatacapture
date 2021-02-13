use structopt::StructOpt;

pub mod api;
mod auth;
pub mod db;
mod error;

use error::Error;

type Result<T> = std::result::Result<T, Error>;

/// opendatacapture
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Database URL.
    #[structopt(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://postgres:postgres@localhost:5432/postgres"
    )]
    pub database_url: String,
    /// Port for the api to listen to.
    #[structopt(long, env = "ODC_API_PORT", default_value = "4321")]
    pub apiport: u16,
    /// Reset the administrative database upon connection.
    #[structopt(long)]
    pub clean: bool,
    /// Email for the first admin user.
    #[structopt(
        long,
        env = "ODC_ADMIN_EMAIL",
        default_value = "admin@example.com"
    )]
    pub admin_email: String,
    /// Password for the first admin user.
    #[structopt(long, env = "ODC_ADMIN_PASSWORD", default_value = "admin")]
    pub admin_password: String,
    /// Prefix for all paths. No prefix is used when this is an empty string.
    #[structopt(long, env = "ODC_API_PREFIX", default_value = "")]
    pub prefix: String,
    /// Disable CORS headers
    #[structopt(long)]
    pub disable_cors: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::user::table::*;
    use crate::db::DB;
    use sqlx::ConnectOptions;

    /// Test database config
    pub fn gen_test_config(dbname: &str) -> db::ConnectionConfig {
        db::ConnectionConfig::new()
            .host("localhost")
            .port(5432)
            .database(dbname)
            .username("odcapi")
            .password("odcapi")
    }

    /// Remove test database
    /// Assumes the postgres database exists - will connect to it with the
    /// same settings as those of `db`
    pub async fn remove_test_db(db: &DB) {
        log::info!("removing test database {}", db.get_name());
        db.get_pool().close().await;
        let config = db.get_config().clone().database("postgres");
        let mut con = config.connect().await.unwrap();
        sqlx::query(
            format!("DROP DATABASE IF EXISTS \"{0}\"", db.get_name()).as_str(),
        )
        .execute(&mut con)
        .await
        .unwrap();
    }

    /// Makes sure the test database database exists.
    /// Assumes the postgres database exists
    pub async fn setup_test_db(dbname: &str) {
        log::info!("setting up database {}", dbname);
        let config = gen_test_config("postgres");
        let mut con = config.connect().await.unwrap();
        sqlx::query(
            format!("DROP DATABASE IF EXISTS \"{0}\"", dbname).as_str(),
        )
        .execute(&mut con)
        .await
        .unwrap();
        sqlx::query(format!("CREATE DATABASE \"{0}\"", dbname).as_str())
            .execute(&mut con)
            .await
            .unwrap();
    }

    /// Create an admin database
    /// If not clean then assume that the database already exists and don't
    /// reset it.
    pub async fn create_test_admindb(
        dbname: &str,
        clean: bool,
        setup: bool,
    ) -> db::admin::AdminDB {
        if setup {
            setup_test_db(dbname).await;
        }
        let mut opt = crate::Opt::from_iter(vec!["appname"]);
        opt.database_url =
            format!("postgres://postgres:postgres@localhost:5432/{}", dbname);
        opt.clean = clean;
        db::admin::AdminDB::new(&opt).await.unwrap()
    }

    /// Insert a test user
    pub async fn insert_test_user(db: &db::admin::AdminDB) {
        db.insert_user("user@example.com", "user", auth::Access::User)
            .await
            .unwrap();
    }

    /// Remove specific databases
    pub async fn remove_dbs(db: &db::admin::AdminDB, names: &[&str]) {
        for name in names {
            sqlx::query(
                format!("DROP DATABASE IF EXISTS \"{}\"", name).as_str(),
            )
            .execute(db.get_pool())
            .await
            .unwrap();
        }
    }

    // Test primary table
    pub fn get_test_primary_table() -> TableMeta {
        let mut cols = ColSpec::new();
        cols.push(
            ColMeta::new()
                .name("id")
                .postgres_type("INTEGER")
                .primary_key(true),
        );
        cols.push(
            ColMeta::new()
                .name("email")
                .postgres_type("TEXT")
                .not_null(true)
                .unique(true),
        );
        TableMeta::new("primary", cols)
    }

    // Test secondary table
    pub fn get_test_secondary_table() -> TableMeta {
        let mut cols = ColSpec::new();
        cols.push(
            ColMeta::new()
                .name("id")
                .postgres_type("INTEGER")
                .primary_key(true)
                .foreign_key(ForeignKey::new("primary", "id")),
        );
        cols.push(
            ColMeta::new()
                .name("timepoint")
                .postgres_type("INTEGER")
                .primary_key(true),
        );
        cols.push(ColMeta::new().name("sick").postgres_type("BOOLEAN"));
        cols.push(ColMeta::new().name("symptoms").postgres_type("JSONB"));
        cols.push(ColMeta::new().name("locations").postgres_type("JSONB"));
        TableMeta::new("secondary", cols)
    }

    // Table with a date column
    pub fn get_date_table() -> TableMeta {
        let mut cols = ColSpec::new();
        cols.push(
            ColMeta::new()
                .name("date")
                .postgres_type("timestamp with time zone"),
        );
        TableMeta::new("timestamptz-table", cols)
    }

    /// Primary table data
    pub fn get_primary_data() -> Vec<RowJson> {
        let mut data = Vec::new();
        let mut row1 = RowJson::new();
        row1.insert("id".to_string(), serde_json::from_str("1").unwrap());
        row1.insert(
            "email".to_string(),
            serde_json::from_str("\"email@example.com\"").unwrap(),
        );
        data.push(row1);
        let mut row2 = RowJson::new();
        row2.insert("id".to_string(), serde_json::from_str("2").unwrap());
        row2.insert(
            "email".to_string(),
            serde_json::from_str("\"email2@example.com\"").unwrap(),
        );
        data.push(row2);
        data
    }

    /// Secondary table partial data
    pub fn get_secondary_data_part() -> Vec<RowJson> {
        let mut data = Vec::new();
        let mut row1 = RowJson::new();
        row1.insert("id".to_string(), serde_json::from_str("1").unwrap());
        row1.insert(
            "timepoint".to_string(),
            serde_json::from_str("1").unwrap(),
        );
        data.push(row1);
        let mut row2 = RowJson::new();
        row2.insert("id".to_string(), serde_json::from_str("1").unwrap());
        row2.insert(
            "timepoint".to_string(),
            serde_json::from_str("2").unwrap(),
        );
        data.push(row2);
        data
    }

    /// Secondary table data with explicit null values
    pub fn get_secondary_data_null() -> Vec<RowJson> {
        let mut data = Vec::new();
        let mut row1 = RowJson::new();
        row1.insert("id".to_string(), serde_json::from_str("1").unwrap());
        row1.insert(
            "timepoint".to_string(),
            serde_json::from_str("3").unwrap(),
        );
        row1.insert("sick".to_string(), serde_json::from_str("null").unwrap());
        row1.insert(
            "symptoms".to_string(),
            serde_json::from_str("null").unwrap(),
        );
        row1.insert(
            "locations".to_string(),
            serde_json::from_str("null").unwrap(),
        );
        data.push(row1);
        data
    }

    /// Secondary table all-column data
    pub fn get_secondary_data() -> Vec<RowJson> {
        let mut data = Vec::new();
        let mut row1 = RowJson::new();
        row1.insert("id".to_string(), serde_json::from_str("2").unwrap());
        row1.insert(
            "timepoint".to_string(),
            serde_json::from_str("1").unwrap(),
        );
        row1.insert("sick".to_string(), serde_json::from_str("false").unwrap());
        row1.insert(
            "symptoms".to_string(),
            serde_json::from_str(r#"{"s1": true, "s2": false}"#).unwrap(),
        );
        row1.insert(
            "locations".to_string(),
            serde_json::from_str(r#"["l1", "l2"]"#).unwrap(),
        );
        data.push(row1);
        let mut row2 = RowJson::new();
        row2.insert("id".to_string(), serde_json::from_str("2").unwrap());
        row2.insert(
            "timepoint".to_string(),
            serde_json::from_str("2").unwrap(),
        );
        row2.insert("sick".to_string(), serde_json::from_str("false").unwrap());
        row2.insert(
            "symptoms".to_string(),
            serde_json::from_str(r#"{"s1": true, "s2": false}"#).unwrap(),
        );
        row2.insert(
            "locations".to_string(),
            serde_json::from_str(r#"["l1", "l2"]"#).unwrap(),
        );
        data.push(row2);
        data
    }

    /// Date data (string, simulate what'll be received in JSON)
    pub fn get_date_data() -> Vec<RowJson> {
        let mut data = Vec::new();
        let mut row1 = RowJson::new();
        row1.insert(
            "date".to_string(),
            serde_json::from_str("\"2020-01-01T00:00:00+00:00\"").unwrap(),
        );
        data.push(row1);
        data
    }
}
