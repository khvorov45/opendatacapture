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
                ON UPDATE CASCADE ON DELETE CASCADE\
            )",
            &[],
        )
        .await?;
        con.execute(
            "CREATE TABLE \"token\" (\
                \"user\" INTEGER NOT NULL,\
                \"token\" TEXT PRIMARY KEY,\
                \"created\" TIMESTAMPTZ NOT NULL,\
                FOREIGN KEY(\"user\") REFERENCES \
                \"user\"(\"id\") \
                ON UPDATE CASCADE ON DELETE CASCADE\
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

    // Access table -----------------------------------------------------------

    /// Fill the access table. Assume that it's empty.
    async fn fill_access(&self) -> Result<()> {
        log::info!("filling presumably empty access table");
        self.get_con()
            .await?
            .execute(
                "INSERT INTO \"access\" (\"access_type\") \
                VALUES ($1), ($2)",
                &[
                    &auth::Access::Admin.to_string(),
                    &auth::Access::User.to_string(),
                ],
            )
            .await?;
        Ok(())
    }

    // User table -------------------------------------------------------------

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
        let admin =
            User::new(admin_email, admin_password, auth::Access::Admin)?;
        self.insert_user(&admin).await?;
        Ok(())
    }
    /// Insert a user
    async fn insert_user(&self, user: &User) -> Result<()> {
        self.get_con().await?.execute(
            "INSERT INTO \"user\" (\"email\", \"access\", \"password_hash\")
            VALUES ($1, $2, $3)",
            &[&user.email, &user.access.to_string(), &user.password_hash],
        ).await?;
        Ok(())
    }
    /// Get all users
    pub async fn get_users(&self) -> Result<Vec<User>> {
        let res = self
            .get_con()
            .await?
            .query("SELECT * FROM \"user\"", &[])
            .await?;
        let mut users = Vec::with_capacity(res.len());
        for row in res {
            users.push(User::from_row(row)?);
        }
        Ok(users)
    }
    /// Returns the user given their token
    pub async fn get_user_by_id(&self, id: i32) -> Result<User> {
        let res = self
            .get_con()
            .await?
            .query_opt("SELECT * FROM \"user\" WHERE \"id\" = $1", &[&id])
            .await?;
        match res {
            Some(row) => User::from_row(row),
            None => Err(Error::NoSuchUser(id.to_string())),
        }
    }
    /// Returns the user for the given email
    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        let res = self
            .get_con()
            .await?
            .query_opt("SELECT * FROM \"user\" WHERE \"email\" = $1", &[&email])
            .await?;
        match res {
            Some(row) => User::from_row(row),
            None => Err(Error::NoSuchUser(email.to_string())),
        }
    }
    /// Gets the user who the given token belongs to
    pub async fn get_user_by_token(&self, tok: &str) -> Result<User> {
        let tok = self.get_token(tok).await?;
        if tok.age_hours() > auth::AUTH_TOKEN_HOURS_TO_LIVE {
            return Err(Error::TokenTooOld);
        }
        self.get_user_by_id(tok.user()).await
    }

    // Token table ------------------------------------------------------------

    /// Get token by the unique string
    async fn get_token(&self, token: &str) -> Result<auth::Token> {
        let res = self
            .get_con()
            .await?
            .query_opt(
                "SELECT * FROM \"token\" WHERE \"token\" = $1",
                &[&token],
            )
            .await?;
        match res {
            Some(row) => Ok(auth::Token::from_row(&row)),
            None => Err(Error::NoSuchToken(token.to_string())),
        }
    }
    /// Inserts a token
    async fn insert_token(&self, tok: &auth::Token) -> Result<()> {
        log::info!("inserting token {:?}", tok);
        self.get_con().await?.execute(
            "INSERT INTO \"token\" (\"user\", \"token\", \"created\") VALUES \
            ($1, $2, $3)",
            &[&tok.user(), tok.token(), tok.created()],
        ).await?;
        Ok(())
    }
    /// Generate a token from email/password combination
    pub async fn generate_session_token(
        &self,
        cred: auth::EmailPassword,
    ) -> Result<auth::Token> {
        let user = self.get_user_by_email(cred.email.as_str()).await?;
        if argon2::verify_encoded(
            user.password_hash.as_str(),
            cred.password.as_bytes(),
        )? {
            let tok = auth::Token::new(user.id);
            self.insert_token(&tok).await?;
            Ok(tok)
        } else {
            Err(Error::WrongPassword(cred.password))
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct User {
    id: i32,
    email: String,
    access: auth::Access,
    password_hash: String,
}

impl User {
    pub fn new(
        email: &str,
        password: &str,
        access: auth::Access,
    ) -> Result<Self> {
        let u = Self {
            id: 1, // Disregard since postgres will handle auto-incrementing
            email: email.to_string(),
            access,
            password_hash: auth::hash(password)?,
        };
        Ok(u)
    }
    pub fn from_row(row: tokio_postgres::Row) -> Result<Self> {
        use std::str::FromStr;
        let u = Self {
            id: row.get("id"),
            email: row.get("email"),
            access: auth::Access::from_str(row.get("access"))?,
            password_hash: row.get("password_hash"),
        };
        Ok(u)
    }
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn email(&self) -> &str {
        self.email.as_str()
    }
    pub fn access(&self) -> auth::Access {
        self.access
    }
    pub fn password_hash(&self) -> &str {
        self.password_hash.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test database config
    fn gen_test_config() -> tokio_postgres::Config {
        let mut pg_config = tokio_postgres::Config::new();
        pg_config
            .host("localhost")
            .port(5432)
            .dbname("odcadmin_test")
            .user("odcapi")
            .password("odcapi");
        pg_config
    }

    // Makes sure odcadmin_test database exists.
    // Assumes odcadmin database exists
    async fn setup_odcadmin_test() {
        let mut config = gen_test_config();
        config.dbname("odcadmin");
        let (odcadmin_client, con) =
            config.connect(tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(async move {
            con.await.unwrap();
        });
        odcadmin_client
            .execute("DROP DATABASE IF EXISTS odcadmin_test", &[])
            .await
            .unwrap();
        odcadmin_client
            .execute("CREATE DATABASE odcadmin_test", &[])
            .await
            .unwrap();
    }

    // Create a database
    async fn test_create(clean: bool) -> AdminDB {
        let pg_config = gen_test_config();
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
                "SELECT \"id\", \"email\", \"password_hash\", \"access\"\
                FROM \"user\" \
                WHERE \"id\" = '1'",
                &[],
            )
            .await
            .unwrap();
        let user = User::from_row(res).unwrap();
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
    async fn gen_tok(db: &AdminDB) -> auth::Token {
        log::info!("verifying that admin@example.com password is admin");
        db.generate_session_token(auth::EmailPassword {
            email: "admin@example.com".to_string(),
            password: "admin".to_string(),
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test() {
        let _ = pretty_env_logger::try_init();
        setup_odcadmin_test().await;
        // Start clean
        log::info!("start clean");
        let test_db = test_create(true).await;
        let user1 = extract_first_user(&test_db).await;
        let tok1 = gen_tok(&test_db).await;
        assert_eq!(extract_first_user_token(&test_db).await, tok1);
        // Restart
        log::info!("restart");
        let test_db = test_create(false).await;
        let user2 = extract_first_user(&test_db).await;
        let tok2 = extract_first_user_token(&test_db).await;
        assert_eq!(user1, user2);
        assert_eq!(tok1, tok2);
        let tok2_reset = gen_tok(&test_db).await;
        assert_ne!(tok2.token(), tok2_reset.token());
        // Start clean again
        log::info!("start clean again");
        let test_db = test_create(true).await;
        let user3 = extract_first_user(&test_db).await;
        assert_ne!(user1.password_hash, user3.password_hash); // Different salt
        let tok3 = gen_tok(&test_db).await;
        assert_ne!(tok3.token(), tok2_reset.token());
        // Insert a regular user
        test_db
            .insert_user(
                &User::new("user@example.com", "user", auth::Access::User)
                    .unwrap(),
            )
            .await
            .unwrap();
        let user_tok = test_db
            .generate_session_token(auth::EmailPassword {
                email: "user@example.com".to_string(),
                password: "user".to_string(),
            })
            .await
            .unwrap();
        let user = test_db.get_user_by_token(user_tok.token()).await.unwrap();
        assert_eq!(user.id, user_tok.user());
    }
}
