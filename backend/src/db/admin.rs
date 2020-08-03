use crate::db::{create_pool, DBPool, DB};
use crate::{auth, Error, Opt, Result};

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
        let con = self.get_con().await?;
        con.execute(
            "CREATE TABLE \"access\" (\"access_type\" TEXT PRIMARY KEY)",
            &[],
        )
        .await?;
        con.execute(
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
        con.execute(
            "CREATE TABLE \"token\" (\
                    \"user\" INTEGER NOT NULL,\
                    \"token\" TEXT NOT NULL,\
                    \"created\" TIMESTAMPTZ NOT NULL,\
                    PRIMARY KEY(\"user\", \"created\"),
                    FOREIGN KEY(\"user\") REFERENCES \
                    \"user\"(\"id\") \
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
        let admin_password_hash = auth::hash(admin_password)?;
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
    ) -> Result<auth::Outcome> {
        match self.get_user(cred.email.as_str()).await {
            Ok(user) => {
                if argon2::verify_encoded(
                    user.password_hash.as_str(),
                    cred.password.as_bytes(),
                )? {
                    let tok = auth::Token::new(user.id);
                    self.insert_token(&tok).await?;
                    Ok(auth::Outcome::Ok(tok))
                } else {
                    Ok(auth::Outcome::Wrong)
                }
            }
            Err(e) => {
                if let Error::NoSuchUser(_) = e {
                    Ok(auth::Outcome::IdNotFound)
                } else {
                    Err(e)
                }
            }
        }
    }
    /// Returns the user for the given email
    async fn get_user(&self, email: &str) -> Result<User> {
        let res = self
            .get_con()
            .await?
            .query_opt(
                "SELECT \"id\", \"email\", \"password_hash\" \
                FROM \"user\" WHERE \"email\" = $1",
                &[&email],
            )
            .await?;
        match res {
            Some(row) => Ok(User {
                id: row.get(0),
                email: row.get(1),
                password_hash: row.get(2),
            }),
            None => Err(Error::NoSuchUser(email.to_string())),
        }
    }
    /// Inserts a token
    async fn insert_token(&self, tok: &auth::Token) -> Result<()> {
        log::info!("inserting token {:?}", tok);
        self.get_con().await?.execute(
            "INSERT INTO \"token\" (\"user\", \"token\", \"created\") VALUES \
            ($1, $2, $3)",
            &[tok.user(), tok.token(), tok.created()],
        ).await?;
        Ok(())
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct EmailPassword {
    email: String,
    password: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct User {
    id: i32,
    email: String,
    password_hash: String,
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

    // Extract first admin
    async fn extract_first_user(db: &AdminDB) -> User {
        let res = db
            .get_con()
            .await
            .unwrap()
            .query_one(
                "SELECT \"id\", \"email\", \"password_hash\" FROM \"user\" \
                WHERE \"id\" = '1'",
                &[],
            )
            .await
            .unwrap();
        let user = User {
            id: res.get(0),
            email: res.get(1),
            password_hash: res.get(2),
        };
        log::info!("first user is {:?}", user);
        user
    }

    // Extract first admin's token
    async fn extract_first_user_token(db: &AdminDB) -> auth::Token {
        let res = db
            .get_con()
            .await
            .unwrap()
            .query_one(
                "SELECT \"user\", \"token\", \"created\" FROM \"token\" \
                WHERE \"user\" = '1'",
                &[],
            )
            .await
            .unwrap();
        let tok = auth::Token::from_row(&res);
        log::info!("first user token is {:?}", tok);
        tok
    }

    // Verify the default password
    async fn verify_password(db: &AdminDB) -> auth::Token {
        log::info!("verifying that admin@example.com password is admin");
        let tok = db
            .authenticate_email_password(EmailPassword {
                email: "admin@example.com".to_string(),
                password: "admin".to_string(),
            })
            .await
            .unwrap();
        assert!(matches!(tok, auth::Outcome::Ok(_)));
        if let auth::Outcome::Ok(tok) = tok {
            tok
        } else {
            panic!("")
        }
    }

    #[tokio::test]
    async fn test() {
        let _ = pretty_env_logger::try_init();
        // Start clean
        log::info!("start clean");
        let test_db = test_create(true).await;
        let user1 = extract_first_user(&test_db).await;
        let tok1 = verify_password(&test_db).await;
        assert_eq!(extract_first_user_token(&test_db).await, tok1);
        // Restart
        log::info!("restart");
        let test_db = test_create(false).await;
        let user2 = extract_first_user(&test_db).await;
        let tok2 = extract_first_user_token(&test_db).await;
        assert_eq!(user1, user2);
        assert_eq!(tok1, tok2);
        let tok2_reset = verify_password(&test_db).await;
        assert_ne!(tok2.token(), tok2_reset.token());
        // Start clean again
        log::info!("start clean again");
        let test_db = test_create(true).await;
        let user3 = extract_first_user(&test_db).await;
        assert_ne!(user1.password_hash, user3.password_hash); // Different salt
        let tok3 = verify_password(&test_db).await;
        assert_ne!(tok3.token(), tok2_reset.token());
    }
}
