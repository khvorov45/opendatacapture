use super::db;

/// Returns a new admin database
pub async fn create_new(
    opt: &super::Opt,
) -> Result<db::DB, Box<dyn std::error::Error>> {
    // Config
    let mut dbconfig = tokio_postgres::config::Config::new();
    dbconfig
        .host(opt.dbhost.as_str())
        .port(opt.dbport)
        .dbname(opt.admindbname.as_str())
        .user(opt.apiusername.as_str())
        .password(opt.apiuserpassword.as_str());
    // Connect to the admin database as the default api user
    let admindb = db::DB::new(&dbconfig, get_tablespec(), !opt.clean).await?;
    // Make sure there is at least one admin
    insert_if_empty(
        &admindb,
        opt.admin_email.as_str(),
        opt.admin_password.as_str(),
    )
    .await?;
    Ok(admindb)
}

/// Tables for the admin database
fn get_tablespec() -> db::TableSpec {
    let mut set = db::TableSpec::new();
    set.push(db::TableMeta::new("admin", get_colspec(), ""));
    set
}

/// Columns for the admin table
fn get_colspec() -> db::ColSpec {
    let mut set = db::ColSpec::new();
    set.push(db::ColMeta::new("id", "SERIAL", "PRIMARY KEY"));
    set.push(db::ColMeta::new("email", "TEXT", "NOT NULL"));
    set.push(db::ColMeta::new("password_hash", "TEXT", ""));
    set
}

/// Insert an admin if the admin table is empty
async fn insert_if_empty(
    admindb: &db::DB,
    admin_email: &str,
    admin_password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !admindb.get_rows_json("admin").await?.is_empty() {
        return Ok(());
    }
    log::info!(
        "no admins found, inserting \"{}\" with password \"{}\"",
        admin_email,
        admin_password
    );
    let admin_password_hash = argon2::hash_encoded(
        admin_password.as_bytes(),
        super::gen_rand_string().as_bytes(),
        &argon2::Config::default(),
    )?;
    let admin_json = format!(
        "{{\"email\": \"{}\", \"password_hash\": \"{}\"}}",
        admin_email, admin_password_hash
    );
    admindb
        .insert(&db::table::TableJson::new(
            "admin",
            vec![serde_json::from_str(admin_json.as_str())?],
        ))
        .await?;
    Ok(())
}
