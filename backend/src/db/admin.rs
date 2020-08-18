use crate::db::{create_pool, DBPool, DB};
use crate::{auth, error::Unauthorized, Error, Opt, Result};

/// Administrative database
pub struct AdminDB {
    name: String,
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
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
    fn get_pool(&self) -> &DBPool {
        &self.pool
    }
    async fn reset(&self) -> Result<()> {
        log::info!("resetting \"{}\" admin database", self.get_name());
        self.remove_all_projects().await?;
        self.drop_all_tables().await?;
        self.create_all_tables().await?;
        Ok(())
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
        con.execute(
            "CREATE TABLE \"project\" (\
                \"user\" INTEGER NOT NULL,\
                \"name\" TEXT PRIMARY KEY,\
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
            name: conf.config.get_dbname().unwrap().to_string(),
            pool: create_pool(conf.config)?,
        };
        // Reset if required
        let connected_to_empty = admindb.is_empty().await?;
        if connected_to_empty {
            admindb.create_all_tables().await?;
        } else if conf.clean {
            admindb.reset().await?;
        }
        // Fill access types and the one admin if required.
        // Also drop all extra databases
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
    pub async fn insert_user(&self, user: &User) -> Result<()> {
        log::info!("inserting user {:?}", user);
        self.get_con().await?.execute(
            "INSERT INTO \"user\" (\"email\", \"access\", \"password_hash\")
            VALUES ($1, $2, $3)",
            &[&user.email, &user.access.to_string(), &user.password_hash],
        ).await?;
        Ok(())
    }
    /// Get all users
    pub async fn get_users(&self) -> Result<Vec<User>> {
        log::debug!("get all users");
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
    /// Returns the user given their id
    pub async fn get_user_by_id(&self, id: i32) -> Result<User> {
        log::debug!("getting user by id: {}", id);
        let res = self
            .get_con()
            .await?
            .query_opt("SELECT * FROM \"user\" WHERE \"id\" = $1", &[&id])
            .await?;
        match res {
            Some(row) => User::from_row(row),
            None => Err(Error::NoSuchUserId(id)),
        }
    }
    /// Returns the user for the given email
    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        log::debug!("getting user by email: {}", email);
        let res = self
            .get_con()
            .await?
            .query_opt("SELECT * FROM \"user\" WHERE \"email\" = $1", &[&email])
            .await?;
        match res {
            Some(row) => User::from_row(row),
            None => Err(Error::NoSuchUserEmail(email.to_string())),
        }
    }
    /// Gets the user who the given token belongs to
    pub async fn get_user_by_token(&self, tok: &str) -> Result<User> {
        log::debug!("getting user by token {}", tok);
        let tok = self.get_token(tok).await?;
        log::debug!("got token: {:?}", tok);
        if tok.age_hours() > auth::AUTH_TOKEN_HOURS_TO_LIVE {
            return Err(Error::Unauthorized(Unauthorized::TokenTooOld));
        }
        // DB guarantees that there will be a user
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
            None => Err(Error::Unauthorized(Unauthorized::NoSuchToken(
                token.to_string(),
            ))),
        }
    }
    /// Inserts a token
    async fn insert_token(&self, tok: &auth::Token) -> Result<()> {
        log::info!("inserting token {:?}", tok);
        self.get_con().await?.execute(
            "INSERT INTO \"token\" (\"user\", \"token\", \"created\") VALUES \
            ($1, $2, $3)",
            &[&tok.user(), &tok.token(), tok.created()],
        ).await?;
        Ok(())
    }
    /// Generate a token from email/password combination
    pub async fn generate_session_token(
        &self,
        cred: auth::EmailPassword,
    ) -> Result<auth::Token> {
        let user;
        match self.get_user_by_email(cred.email.as_str()).await {
            Ok(u) => user = u,
            Err(e) => match e {
                Error::NoSuchUserEmail(email) => {
                    return Err(Error::Unauthorized(
                        Unauthorized::NoSuchUserEmail(email),
                    ))
                }
                _ => return Err(e),
            },
        };
        if argon2::verify_encoded(
            user.password_hash.as_str(),
            cred.password.as_bytes(),
        )? {
            let tok = auth::Token::new(user.id);
            self.insert_token(&tok).await?;
            Ok(tok)
        } else {
            Err(Error::Unauthorized(Unauthorized::WrongPassword(
                cred.password,
            )))
        }
    }

    // Project table ----------------------------------------------------------

    /// Create a project
    pub async fn create_project(
        &self,
        user_id: i32,
        project_name: &str,
    ) -> Result<()> {
        let project = Project::new(user_id, project_name);
        // Create the database
        self.get_con()
            .await?
            .execute(
                format!("CREATE DATABASE \"{}\"", project.name).as_str(),
                &[],
            )
            .await?;
        // Insert a record of it into the project table
        self.insert_project(&project).await?;
        Ok(())
    }
    /// Insert an entry into the project table
    async fn insert_project(&self, project: &Project) -> Result<()> {
        self.get_con()
            .await?
            .execute(
                "INSERT INTO \"project\" (\"user\", \"name\", \"created\") \
                VALUES ($1, $2, $3)",
                &[&project.user, &project.name, &project.created],
            )
            .await?;
        Ok(())
    }
    /// Removes the given project including dropping the database
    pub async fn remove_project(&self, project: &Project) -> Result<()> {
        // Drop the database
        self.get_con()
            .await?
            .execute(
                format!("DROP DATABASE \"{}\"", project.name).as_str(),
                &[],
            )
            .await?;
        // Delete the record
        self.delete_project(&project).await?;
        Ok(())
    }
    /// Delete an entry from a project table
    async fn delete_project(&self, project: &Project) -> Result<()> {
        self.get_con()
            .await?
            .execute(
                "DELETE FROM \"project\" WHERE \"name\" = $1",
                &[&project.name],
            )
            .await?;
        Ok(())
    }
    /// Removes all projects
    pub async fn remove_all_projects(&self) -> Result<()> {
        log::info!("removing all projects");
        let all_projects = self.get_all_projects().await?;
        for project in &all_projects {
            self.remove_project(project).await?;
        }
        Ok(())
    }
    /// Returns all projects
    pub async fn get_project(
        &self,
        user_id: i32,
        project_name: &str,
    ) -> Result<Project> {
        let project = Project::new(user_id, project_name);
        let res = self
            .get_con()
            .await?
            .query_opt(
                "SELECT * FROM \"project\" WHERE \"name\" = $1",
                &[&project.name],
            )
            .await?;
        match res {
            None => {
                Err(Error::NoSuchProject(user_id, project_name.to_string()))
            }
            Some(row) => Ok(Project::from_row(row)),
        }
    }
    /// Returns all projects
    pub async fn get_all_projects(&self) -> Result<Vec<Project>> {
        let res = self
            .get_con()
            .await?
            .query("SELECT * FROM \"project\"", &[])
            .await?;
        let mut projects = Vec::<Project>::with_capacity(res.len());
        for row in res {
            projects.push(Project::from_row(row))
        }
        Ok(projects)
    }
    /// Returns user's projects
    pub async fn get_user_projects(
        &self,
        user_id: i32,
    ) -> Result<Vec<Project>> {
        log::debug!("getting user projects");
        let res = self
            .get_con()
            .await?
            .query("SELECT * FROM \"project\" WHERE \"user\" = $1", &[&user_id])
            .await?;
        let mut projects = Vec::<Project>::with_capacity(res.len());
        for row in res {
            projects.push(Project::from_row(row))
        }
        log::debug!("got projects: {:?}", projects);
        Ok(projects)
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

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
pub struct Project {
    user: i32,
    name: String,
    created: chrono::DateTime<chrono::Utc>,
}

impl Project {
    pub fn new(user: i32, project_name: &str) -> Self {
        Self {
            user,
            name: format!("user{}_{}", user, project_name),
            created: chrono::Utc::now(),
        }
    }
    pub fn from_row(row: tokio_postgres::Row) -> Self {
        Self {
            user: row.get("user"),
            name: row.get("name"),
            created: row.get("created"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        db.generate_session_token(auth::EmailPassword {
            email: "admin@example.com".to_string(),
            password: "admin".to_string(),
        })
        .await
        .unwrap()
    }

    /// Get a list of all database names
    async fn get_db_list(db: &AdminDB) -> Vec<String> {
        let db_list_query = db
            .get_con()
            .await
            .unwrap()
            .query("SELECT datname FROM pg_database;", &[])
            .await
            .unwrap();
        let mut db_list = Vec::<String>::with_capacity(db_list_query.len());
        for db_list_query_row in db_list_query {
            db_list.push(db_list_query_row.get(0));
        }
        db_list
    }

    /// Verify that a project exists
    async fn project_exists(db: &AdminDB, name: &str) -> bool {
        let db_exists = get_db_list(&db).await.contains(&name.to_string());
        let project_exists = db
            .get_all_projects()
            .await
            .unwrap()
            .iter()
            .any(|p| p.name == name);
        assert_eq!(db_exists, project_exists);
        db_exists
    }

    #[tokio::test]
    async fn test_admin() {
        let _ = pretty_env_logger::try_init();

        // Start clean
        log::info!("start clean");
        let test_db = crate::tests::create_test_admindb(
            "odcadmin_test_admin",
            true,
            true,
        )
        .await;
        // Generate token
        let user1 = extract_first_user(&test_db).await;
        let tok1 = gen_tok(&test_db).await;
        assert_eq!(extract_first_user_token(&test_db).await, tok1);

        // Restart
        log::info!("restart");
        drop(test_db);
        let test_db = crate::tests::create_test_admindb(
            "odcadmin_test_admin",
            false,
            false,
        )
        .await;
        // Data should not be modified
        let user2 = extract_first_user(&test_db).await;
        let tok2 = extract_first_user_token(&test_db).await;
        assert_eq!(user1, user2);
        assert_eq!(tok1, tok2);
        // Next generated token should be different
        let tok2_next = gen_tok(&test_db).await;
        assert_ne!(tok2.token(), tok2_next.token());

        // Start clean again
        log::info!("start clean again");
        drop(test_db);
        let test_db = crate::tests::create_test_admindb(
            "odcadmin_test_admin",
            true,
            false,
        )
        .await;
        // Password hash should be different
        let user3 = extract_first_user(&test_db).await;
        assert_eq!(user3.id(), user1.id());
        assert_ne!(user1.password_hash, user3.password_hash); // Different salt

        // Token should be absent
        let res = test_db
            .get_con()
            .await
            .unwrap()
            .query_opt("SELECT * FROM \"token\"", &[])
            .await
            .unwrap();
        assert!(res.is_none());

        // Insert a regular user
        crate::tests::insert_test_user(&test_db).await;
        // Token should be successfully generated
        let user_tok = test_db
            .generate_session_token(auth::EmailPassword {
                email: "user@example.com".to_string(),
                password: "user".to_string(),
            })
            .await
            .unwrap();
        let user = test_db.get_user_by_token(user_tok.token()).await.unwrap();
        assert_eq!(user.id, user_tok.user());

        // Make that token appear older
        test_db
            .get_con()
            .await
            .unwrap()
            .execute(
                "UPDATE \"token\" \
                SET \"created\" = '2000-08-14 08:15:29.425665+10' \
                WHERE \"user\" = '2'",
                &[],
            )
            .await
            .unwrap();
        let user = test_db.get_user_by_token(user_tok.token()).await;
        assert!(matches!(
            user,
            Err(Error::Unauthorized(Unauthorized::TokenTooOld))
        ));

        // User 3 should not exist
        let user3 = test_db.get_user_by_id(3).await;
        assert!(matches!(user3, Err(Error::NoSuchUserId(id)) if id == 3));
        let user3 = test_db.get_user_by_email("user3@email.com").await;
        assert!(matches!(
            user3,
            Err(Error::NoSuchUserEmail(email)) if email == "user3@email.com"
        ));
        let user3 = test_db.get_user_by_token("abc").await;
        assert!(matches!(
            user3,
            Err(Error::Unauthorized(Unauthorized::NoSuchToken(tok)))
                if tok == "abc"
        ));

        // Project creation/removal -------------------------------------------

        // Verify that the database does not exist
        assert!(!project_exists(&test_db, "user1_test").await);

        // Create project
        test_db.create_project(1, "test").await.unwrap();

        // Verify that the database was created
        assert!(project_exists(&test_db, "user1_test").await);
        let project = test_db.get_project(1, "test").await.unwrap();
        assert_eq!(project.name, "user1_test");
        let err = test_db.get_project(1, "test1").await.unwrap_err();
        assert!(matches!(
                err,
                Error::NoSuchProject(id, name) if id == 1 && name == "test1"));

        // Reconnect
        drop(test_db);
        let test_db = crate::tests::create_test_admindb(
            "odcadmin_test_admin",
            false,
            false,
        )
        .await;
        assert!(project_exists(&test_db, "user1_test").await);

        // Reconnect cleanly
        drop(test_db);
        let test_db = crate::tests::create_test_admindb(
            "odcadmin_test_admin",
            true,
            false,
        )
        .await;
        assert!(!project_exists(&test_db, "user1_test").await);

        // Create a project as a different user
        assert!(!project_exists(&test_db, "user2_test").await);
        test_db.create_project(1, "test").await.unwrap();
        assert!(project_exists(&test_db, "user1_test").await);
        crate::tests::insert_test_user(&test_db).await;
        test_db.create_project(2, "test").await.unwrap();
        assert!(project_exists(&test_db, "user2_test").await);
        assert_eq!(test_db.get_all_projects().await.unwrap().len(), 2);
        assert_eq!(test_db.get_user_projects(2).await.unwrap().len(), 1);
        test_db.remove_all_projects().await.unwrap();
        assert!(!project_exists(&test_db, "user2_test").await);
        assert!(!project_exists(&test_db, "user1_test").await);
    }
}
