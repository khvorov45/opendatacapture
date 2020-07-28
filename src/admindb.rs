use super::{db, Opt};
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Administrative database
pub struct AdminDB {
    db: db::DB,
}

impl AdminDB {
    pub async fn new(opt: &Opt) -> Result<Self> {
        // Config
        let mut dbconfig = tokio_postgres::config::Config::new();
        dbconfig
            .host(opt.dbhost.as_str())
            .port(opt.dbport)
            .dbname(opt.admindbname.as_str())
            .user(opt.apiusername.as_str())
            .password(opt.apiuserpassword.as_str());
        // Connect to the admin database as the default api user
        let admindb = Self {
            db: db::DB::new(&dbconfig, get_tablespec()).await?,
        };
        // Reset if required
        if opt.clean && !admindb.db.was_empty() {
            admindb.db.reset(false).await?;
        }
        // Fill access types and the one admin if required
        if opt.clean || admindb.db.was_empty() {
            admindb.fill_access().await?;
            admindb
                .insert_admin(
                    opt.admin_email.as_str(),
                    opt.admin_password.as_str(),
                )
                .await?;
        }
        Ok(admindb)
    }
    /// Fill the access table. Assume that it's empty.
    async fn fill_access(&self) -> Result<()> {
        log::info!("filling presumably empty access table");
        // Needed entries as json values
        let access_types: Vec<serde_json::Value> = vec!["admin", "user"]
            .iter()
            .map(|t| serde_json::json!(t))
            .collect();
        // Entries as a vector of rows
        let mut access_entries =
            Vec::<db::RowJson>::with_capacity(access_types.len());
        for access_type in access_types {
            let mut access_entry = serde_json::Map::new();
            access_entry.insert(String::from("access_type"), access_type);
            access_entries.push(access_entry)
        }
        self.db
            .insert(&db::TableJson::new("access", access_entries))
            .await?;
        Ok(())
    }
    /// Insert an admin. Assume the admin table is empty.
    async fn insert_admin(
        &self,
        admin_email: &str,
        admin_password: &str,
    ) -> Result<()> {
        log::info!(
            "inserting admin \"{}\" with password \"{}\"",
            admin_email,
            admin_password
        );
        let admin_password_hash = super::password::hash(admin_password)?;
        let admin_json = format!(
            "{{\
            \"email\": \"{}\",
            \"access\": \"admin\",\
            \"password_hash\": \"{}\"\
        }}",
            admin_email, admin_password_hash
        );
        self.db
            .insert(&db::table::TableJson::new(
                "user",
                vec![serde_json::from_str(admin_json.as_str())?],
            ))
            .await?;
        Ok(())
    }
}

/// Tables for the admin database
fn get_tablespec() -> db::TableSpec {
    let mut set = db::TableSpec::new();
    set.push(db::TableMeta::new("access", get_access_colspec(), ""));
    set.push(db::TableMeta::new(
        "user",
        get_user_colspec(),
        "FOREIGN KEY(access) REFERENCES access(access_type) \
        ON UPDATE CASCADE ON DELETE CASCADE",
    ));
    set
}

/// Columns for the access table
fn get_access_colspec() -> db::ColSpec {
    let mut set = db::ColSpec::new();
    set.push(db::ColMeta::new("access_type", "TEXT", "PRIMARY KEY"));
    set
}

/// Columns for the user table
fn get_user_colspec() -> db::ColSpec {
    let mut set = db::ColSpec::new();
    set.push(db::ColMeta::new("id", "SERIAL", "PRIMARY KEY"));
    set.push(db::ColMeta::new("email", "TEXT", "NOT NULL UNIQUE"));
    set.push(db::ColMeta::new("access", "TEXT", "NOT NULL"));
    set.push(db::ColMeta::new("password_hash", "TEXT", "NOT NULL"));
    set
}

pub mod error {
    /// Handler errors
    #[derive(thiserror::Error, Debug)]
    pub enum Error {
        /// Argon errors
        #[error(transparent)]
        Argon(#[from] argon2::Error),
        /// Database errors
        #[error(transparent)]
        DB(#[from] super::db::Error),
        /// serde_json errors
        #[error(transparent)]
        SerdeJson(#[from] serde_json::Error),
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Opt, StructOpt};
    use super::*;

    // Create a database
    async fn test_create(clean: bool) -> AdminDB {
        let mut args = vec!["appname", "--admindbname", "odcadmin_test"];
        if clean {
            args.push("--clean")
        }
        let opt = Opt::from_iter(args);
        let test_admin_db = AdminDB::new(&opt).await.unwrap();
        // Clean or not, there should be one row in the user table
        assert_eq!(
            test_admin_db
                .db
                .select("user", &[], "", &[])
                .await
                .unwrap()
                .len(),
            1
        );
        test_admin_db
    }

    // Extract first admin's hash
    async fn extract_first_user_hash(db: &AdminDB) -> String {
        let user_rows = db.db.select("user", &[], "", &[]).await.unwrap();
        if let serde_json::Value::String(hash) = &user_rows[0]["password_hash"]
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
        let hash1 = extract_first_user_hash(&test_db).await;
        // Restart with backup
        let test_db = test_create(false).await;
        let hash2 = extract_first_user_hash(&test_db).await;
        assert_eq!(hash1, hash2);
        // Start clean again
        let test_db = test_create(true).await;
        let hash3 = extract_first_user_hash(&test_db).await;
        assert_ne!(hash1, hash3);
    }
}
