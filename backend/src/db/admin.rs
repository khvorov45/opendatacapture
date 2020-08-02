use super::{create_pool, password, DBPool, Error, Opt, Result, DB};

/// Administrative database
pub struct AdminDB {
    pool: DBPool,
}

/// Administrative database config
pub struct Config {
    pub config: tokio_postgres::Config,
    pub clean: bool,
    pub admin_email: String,
    pub admin_password: String,
}

impl Config {
    pub fn from_opts(opt: &Opt) -> Self {
        let mut config = tokio_postgres::config::Config::new();
        config
            .host(opt.dbhost.as_str())
            .port(opt.dbport)
            .dbname(opt.admindbname.as_str())
            .user(opt.apiusername.as_str())
            .password(opt.apiuserpassword.as_str());
        Self {
            config,
            clean: opt.clean,
            admin_email: opt.admin_email.to_string(),
            admin_password: opt.admin_password.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl DB for AdminDB {
    fn get_name(&self) -> String {
        "admin".to_string()
    }
    fn get_pool(&self) -> &DBPool {
        &self.pool
    }
    async fn create_all_tables(&self) -> Result<()> {
        self.get_con()
            .await?
            .execute(
                "CREATE TABLE \"access\" (\"access_type\" TEXT PRIMARY KEY)",
                &[],
            )
            .await?;
        self.get_con()
            .await?
            .execute(
                "CREATE TABLE \"user\" (\
                    \"id\" SERIAL PRIMARY KEY,\
                    \"email\" TEXT NOT NULL UNIQUE,\
                    \"access\" TEXT NOT NULL,\
                    \"password_hash\" TEXT NOT NULL,\
                    FOREIGN KEY(\"access\") REFERENCES \
                    \"access\"(\"access_type\") \
                    ON UPDATE CASCADE ON DELETE CASCADE
                )",
                &[],
            )
            .await?;
        Ok(())
    }
}

impl AdminDB {
    pub async fn new(conf: Config) -> Result<Self> {
        // Connect to the admin database as the default api user
        let admindb = Self {
            pool: create_pool(conf.config)?,
        };
        // Reset if required
        let connected_to_empty = admindb.is_empty().await?;
        if connected_to_empty {
            admindb.create_all_tables().await?;
        } else if conf.clean {
            admindb.reset().await?;
        }
        // Fill access types and the one admin if required
        if conf.clean || connected_to_empty {
            admindb.fill_access().await?;
            admindb
                .insert_admin(
                    conf.admin_email.as_str(),
                    conf.admin_password.as_str(),
                )
                .await?;
        }
        Ok(admindb)
    }
    /// Fill the access table. Assume that it's empty.
    async fn fill_access(&self) -> Result<()> {
        log::info!("filling presumably empty access table");
        self.get_con()
            .await?
            .execute(
                "INSERT INTO \"access\" (\"access_type\") \
                VALUES ('admin'), ('user')",
                &[],
            )
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
        let admin_password_hash = password::hash(admin_password)?;
        self.get_con()
            .await?
            .execute(
                format!(
                    "INSERT INTO \"user\" \
                    (\"email\", \"access\", \"password_hash\") \
                    VALUES ('{}', 'admin', '{}')",
                    admin_email, admin_password_hash
                )
                .as_str(),
                &[],
            )
            .await?;
        Ok(())
    }
    /// Authenticates an email/password combination
    pub async fn authenticate_email_password(
        &self,
        cred: EmailPassword,
    ) -> Result<bool> {
        let hash = self.get_password_hash(cred.email.as_str()).await?;
        let res =
            argon2::verify_encoded(hash.as_str(), cred.password.as_bytes())?;
        Ok(res)
    }
    /// Returns the password hash for the given email
    async fn get_password_hash(&self, email: &str) -> Result<String> {
        let res = self
            .get_con()
            .await?
            .query_opt(
                "SELECT \"password_hash\" FROM \"user\" WHERE \"email\" = $1",
                &[&email],
            )
            .await?;
        match res {
            Some(row) => Ok(row.get(0)),
            None => Err(Error::NoSuchUser(email.to_string())),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct EmailPassword {
    email: String,
    password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Create a database
    async fn test_create(clean: bool) -> AdminDB {
        let mut pg_config = tokio_postgres::Config::new();
        pg_config
            .host("localhost")
            .port(5432)
            .dbname("odcadmin_test")
            .user("odcapi")
            .password("odcapi");
        let admin_conf = Config {
            config: pg_config,
            clean,
            admin_email: "admin@example.com".to_string(),
            admin_password: "admin".to_string(),
        };
        let test_admin_db = AdminDB::new(admin_conf).await.unwrap();
        // Clean or not, there should be one row in the user table
        assert_eq!(
            test_admin_db
                .get_con()
                .await
                .unwrap()
                .query("SELECT * FROM \"user\"", &[])
                .await
                .unwrap()
                .len(),
            1
        );
        test_admin_db
    }

    // Extract first admin's hash
    async fn extract_first_user_hash(db: &AdminDB) -> String {
        db.get_con()
            .await
            .unwrap()
            .query_one(
                "SELECT \"password_hash\" FROM \"user\" \
                WHERE \"id\" = '1'",
                &[],
            )
            .await
            .unwrap()
            .get(0)
    }

    // Verify the default password
    async fn verify_password(db: &AdminDB) {
        assert!(db
            .authenticate_email_password(EmailPassword {
                email: "admin@example.com".to_string(),
                password: "admin".to_string()
            })
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test() {
        let _ = pretty_env_logger::try_init();
        // Start clean
        let test_db = test_create(true).await;
        let hash1 = extract_first_user_hash(&test_db).await;
        verify_password(&test_db).await;
        // Restart
        let test_db = test_create(false).await;
        let hash2 = extract_first_user_hash(&test_db).await;
        assert_eq!(hash1, hash2);
        // Start clean again
        let test_db = test_create(true).await;
        let hash3 = extract_first_user_hash(&test_db).await;
        assert_ne!(hash1, hash3);
        verify_password(&test_db).await;
    }
}
