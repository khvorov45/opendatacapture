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
    let admin_password_hash = super::password::hash(admin_password)?;
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

#[cfg(test)]
mod tests {
    use super::super::{Opt, StructOpt};
    use super::*;

    // Create a database
    async fn test_create(clean: bool) -> db::DB {
        let mut args = vec!["appname", "--admindbname", "odcadmin_test"];
        if clean {
            args.push("--clean")
        }
        let opt = Opt::from_iter(args);
        let test_admin_db = create_new(&opt).await.unwrap();
        // Clean or not, there should be one row in the admin table
        assert_eq!(
            test_admin_db.get_rows_json("admin").await.unwrap().len(),
            1
        );
        test_admin_db
    }

    // Extract first admin's hash
    async fn extract_first_admin_hash(db: &db::DB) -> String {
        let admin_rows = db.get_rows_json("admin").await.unwrap();
        if let serde_json::Value::String(hash) = &admin_rows[0]["password_hash"]
        {
            String::from(hash)
        } else {
            panic!("unexpected lack of string")
        }
    }

    #[tokio::test]
    async fn test() {
        let _ = pretty_env_logger::try_init();
        // Start clean
        let test_db = test_create(true).await;
        let hash1 = extract_first_admin_hash(&test_db).await;
        // Restart with backup
        let test_db = test_create(false).await;
        let hash2 = extract_first_admin_hash(&test_db).await;
        assert_eq!(hash1, hash2);
        // Start clean again
        let test_db = test_create(true).await;
        let hash3 = extract_first_admin_hash(&test_db).await;
        assert_ne!(hash1, hash3);
    }
}
