use crate::db::{user, Database, PoolMeta, DB};
use crate::{auth, error::Unauthorized, Error, Result};
use user::table::{RowJson, TableMeta, TableSpec};
use user::UserDB;

/// Administrative database
pub struct AdminDB {
    pool: PoolMeta,
    user_dbs: Vec<UserDB>,
}

#[async_trait::async_trait]
impl DB for AdminDB {
    fn get_pool_meta(&self) -> &PoolMeta {
        &self.pool
    }
}

impl AdminDB {
    pub async fn new(opt: &crate::Opt) -> Result<Self> {
        // Connect to the admin database as the default api user
        let mut admindb = Self {
            pool: PoolMeta::from_opt(&opt).await?,
            user_dbs: Vec::new(),
        };
        // Reset if required
        let connected_to_empty = admindb.is_empty().await?;
        if connected_to_empty {
            admindb.create_all_tables().await?;
        } else if opt.clean {
            admindb.reset().await?;
        }
        // Fill access types and the one admin if required.
        if opt.clean || connected_to_empty {
            admindb
                .insert_admin(
                    opt.admin_email.as_str(),
                    opt.admin_password.as_str(),
                )
                .await?;
        }
        Ok(admindb)
    }

    /// Resets the database
    async fn reset(&mut self) -> Result<()> {
        log::info!("resetting \"{}\" admin database", self.get_name());
        if self
            .get_all_table_names()
            .await?
            .contains(&"project".to_string())
        {
            self.remove_all_projects().await?;
        }
        self.drop_all_tables().await?;
        self.create_all_tables().await?;
        Ok(())
    }

    /// Locate user db in the vector
    /// If not present, will append an entry.
    async fn get_user_db(&mut self, project: &Project) -> Result<&UserDB> {
        let name = project.get_dbname(self.get_name());
        if let Some(i) =
            self.user_dbs.iter().position(|db| db.get_name() == name)
        {
            return Ok(&self.user_dbs[i]);
        };
        let db = UserDB::new(self.get_config(), name.as_str()).await?;
        self.user_dbs.push(db);
        Ok(&self.user_dbs[self.user_dbs.len() - 1])
    }

    /// Creates tables
    async fn create_all_tables(&self) -> Result<()> {
        sqlx::query("DROP TYPE IF EXISTS odc_user_access")
            .execute(self.get_pool())
            .await?;
        sqlx::query("CREATE TYPE odc_user_access AS ENUM ('User', 'Admin')")
            .execute(self.get_pool())
            .await?;
        sqlx::query(
            "CREATE TABLE \"user\" (\
                \"id\" SERIAL PRIMARY KEY,\
                \"email\" TEXT NOT NULL UNIQUE,\
                \"access\" odc_user_access NOT NULL,\
                \"password_hash\" TEXT NOT NULL\
            )",
        )
        .execute(self.get_pool())
        .await?;
        sqlx::query(
            "CREATE TABLE \"token\" (\
                \"user\" INTEGER NOT NULL,\
                \"token\" TEXT PRIMARY KEY,\
                \"created\" TIMESTAMPTZ NOT NULL,\
                FOREIGN KEY(\"user\") REFERENCES \
                \"user\"(\"id\") \
                ON UPDATE CASCADE ON DELETE CASCADE\
            )",
        )
        .execute(self.get_pool())
        .await?;
        sqlx::query(
            "CREATE TABLE \"project\" (\
                \"user\" INTEGER,\
                \"name\" TEXT,\
                \"created\" TIMESTAMPTZ NOT NULL,\
                PRIMARY KEY(\"user\", \"name\"),
                FOREIGN KEY(\"user\") REFERENCES \
                \"user\"(\"id\") \
                ON UPDATE CASCADE ON DELETE CASCADE\
            )",
        )
        .execute(self.get_pool())
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
        sqlx::query(
            "INSERT INTO \"user\" (\"email\", \"access\", \"password_hash\")
            VALUES ($1, $2, $3)",
        )
        .bind(user.email.as_str())
        .bind(user.access)
        .bind(user.password_hash.as_str())
        .execute(self.get_pool())
        .await?;
        Ok(())
    }
    /// Get all users
    pub async fn get_users(&self) -> Result<Vec<User>> {
        log::debug!("get all users");
        let users = sqlx::query_as::<Database, User>("SELECT * FROM \"user\"")
            .fetch_all(self.get_pool())
            .await?;
        Ok(users)
    }
    /// Returns the user given their id
    pub async fn get_user_by_id(&self, id: i32) -> Result<User> {
        log::debug!("getting user by id: {}", id);
        let res = sqlx::query_as::<Database, User>(
            "SELECT * FROM \"user\" WHERE \"id\" = $1",
        )
        .bind(id)
        .fetch_optional(self.get_pool())
        .await?;
        match res {
            Some(user) => Ok(user),
            None => Err(Error::NoSuchUserId(id)),
        }
    }
    /// Returns the user for the given email
    async fn get_user_by_email(&self, email: &str) -> Result<User> {
        log::debug!("getting user by email: {}", email);
        let res = sqlx::query_as::<Database, User>(
            "SELECT * FROM \"user\" WHERE \"email\" = $1",
        )
        .bind(email)
        .fetch_optional(self.get_pool())
        .await?;
        match res {
            Some(user) => Ok(user),
            None => Err(Error::NoSuchUserEmail(email.to_string())),
        }
    }
    /// Gets the user who the given valid token belongs to
    pub async fn get_user_by_token(&self, tok: &str) -> Result<User> {
        log::debug!("getting user by token {}", tok);
        let tok = self.get_token_valid(tok).await?;
        // DB guarantees that there will be a user
        self.get_user_by_id(tok.user()).await
    }

    // Token table ------------------------------------------------------------

    /// Get token by the unique string and makes sure it's valid
    async fn get_token_valid(&self, token: &str) -> Result<auth::Token> {
        let res = sqlx::query_as::<Database, auth::Token>(
            "SELECT * FROM \"token\" WHERE \"token\" = $1",
        )
        .bind(auth::hash_fast(token))
        .fetch_optional(self.get_pool())
        .await?;
        match res {
            Some(tok) => {
                if tok.age_hours() > auth::AUTH_TOKEN_HOURS_TO_LIVE {
                    Err(Error::Unauthorized(Unauthorized::TokenTooOld))
                } else {
                    Ok(tok)
                }
            }
            None => Err(Error::Unauthorized(Unauthorized::NoSuchToken(
                token.to_string(),
            ))),
        }
    }
    /// Inserts a token
    async fn insert_token(&self, tok: &auth::Token) -> Result<()> {
        log::info!("inserting token {:?}", tok);
        sqlx::query(
            "INSERT INTO \"token\" (\"user\", \"token\", \"created\") VALUES \
            ($1, $2, $3)",
        )
        .bind(tok.user())
        .bind(auth::hash_fast(tok.token()))
        .bind(tok.created())
        .execute(self.get_pool())
        .await?;
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
    /// Refresh a token - get valid old and insert and return new
    pub async fn refresh_token(&self, token: &str) -> Result<auth::Token> {
        let old_token = self.get_token_valid(token).await?;
        let new_token = auth::Token::new(old_token.user());
        self.insert_token(&new_token).await?;
        Ok(new_token)
    }
    /// Remove the given token regardless of its validity
    pub async fn remove_token(&self, token: &str) -> Result<()> {
        log::debug!("removing token {}", token);
        sqlx::query("DELETE FROM \"token\" WHERE \"token\" = $1")
            .bind(auth::hash_fast(token))
            .execute(self.get_pool())
            .await?;
        Ok(())
    }

    // Project table ----------------------------------------------------------

    /// Create a project
    pub async fn create_project(
        &self,
        user_id: i32,
        project_name: &str,
    ) -> Result<()> {
        log::debug!(
            "creating project {} for user id {}",
            project_name,
            user_id
        );
        let project = Project::new(user_id, project_name);
        if self.get_project(user_id, project_name).await.is_ok() {
            return Err(Error::ProjectAlreadyExists(
                user_id,
                project_name.to_string(),
            ));
        }
        // Create the database
        sqlx::query(
            format!(
                "CREATE DATABASE \"{}\"",
                project.get_dbname(self.get_name())
            )
            .as_str(),
        )
        .execute(self.get_pool())
        .await?;

        // Insert a record of it into the project table
        self.insert_project(&project).await?;
        Ok(())
    }
    /// Insert an entry into the project table
    async fn insert_project(&self, project: &Project) -> Result<()> {
        sqlx::query(
            "INSERT INTO \"project\" (\"user\", \"name\", \"created\") \
            VALUES ($1, $2, $3)",
        )
        .bind(project.user)
        .bind(project.name.as_str())
        .bind(project.created)
        .execute(self.get_pool())
        .await?;
        Ok(())
    }
    /// Removes the given project including dropping the database
    pub async fn remove_project(
        &mut self,
        user_id: i32,
        project_name: &str,
    ) -> Result<()> {
        log::debug!(
            "removing project {} for user id {}",
            project_name,
            user_id
        );
        let project = self.get_project(user_id, project_name).await?;
        let db_name = project.get_dbname(self.get_name());

        // Remove the entry from UserDBs and close connections
        if let Some(i) =
            self.user_dbs.iter().position(|p| p.get_name() == db_name)
        {
            self.user_dbs.remove(i).get_pool().close().await;
        }

        // Drop the database
        sqlx::query(format!("DROP DATABASE \"{}\"", db_name).as_str())
            .execute(self.get_pool())
            .await?;
        // Delete the record
        self.delete_project(&project).await?;
        Ok(())
    }
    /// Delete an entry from a project table
    async fn delete_project(&self, project: &Project) -> Result<()> {
        log::info!("deleting project {:?}", project);
        sqlx::query(
            "DELETE FROM \"project\" WHERE \"name\" = $1 AND \"user\" = $2",
        )
        .bind(project.name.as_str())
        .bind(project.user)
        .execute(self.get_pool())
        .await?;
        Ok(())
    }
    /// Removes all projects
    pub async fn remove_all_projects(&mut self) -> Result<()> {
        log::info!("removing all projects");
        let all_projects = self.get_all_projects().await?;
        for project in &all_projects {
            self.remove_project(project.user, project.name.as_str())
                .await?;
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
        let res = sqlx::query_as::<Database, Project>(
            "SELECT * FROM \"project\" WHERE \"name\" = $1 AND \"user\" = $2",
        )
        .bind(project.name)
        .bind(project.user)
        .fetch_optional(self.get_pool())
        .await?;
        match res {
            None => {
                Err(Error::NoSuchProject(user_id, project_name.to_string()))
            }
            Some(project) => Ok(project),
        }
    }
    /// Returns all projects
    pub async fn get_all_projects(&self) -> Result<Vec<Project>> {
        let projects =
            sqlx::query_as::<Database, Project>("SELECT * FROM \"project\"")
                .fetch_all(self.get_pool())
                .await?;
        Ok(projects)
    }
    /// Returns user's projects
    pub async fn get_user_projects(
        &self,
        user_id: i32,
    ) -> Result<Vec<Project>> {
        log::debug!("getting user id {} projects", user_id);
        let projects = sqlx::query_as::<Database, Project>(
            "SELECT * FROM \"project\" WHERE \"user\" = $1",
        )
        .bind(user_id)
        .fetch_all(self.get_pool())
        .await?;
        log::debug!("got projects: {:?}", projects);
        Ok(projects)
    }
    /// Returns one project
    pub async fn get_user_project(
        &self,
        user_id: i32,
        project_name: &str,
    ) -> Result<Project> {
        log::debug!("getting user id {} project {}", user_id, project_name);
        let res = sqlx::query_as::<Database, Project>(
            "SELECT * FROM \"project\" WHERE \"user\" = $1 AND \"name\" = $2",
        )
        .bind(user_id)
        .bind(project_name)
        .fetch_optional(self.get_pool())
        .await?;
        match res {
            Some(project) => {
                log::debug!("got project: {:?}", project);
                Ok(project)
            }
            None => {
                Err(Error::NoSuchProject(user_id, project_name.to_string()))
            }
        }
    }

    // Project manipulation ---------------------------------------------------

    /// Creates a table in a user's database
    pub async fn create_user_table(
        &mut self,
        project: &Project,
        table: &TableMeta,
    ) -> Result<()> {
        let db_name = project.get_dbname(self.get_name());
        log::debug!("creating table {} in database {}", table.name, db_name);
        self.get_user_db(project).await?.create_table(table).await
    }
    /// Removes a table from a user's database
    pub async fn remove_user_table(
        &mut self,
        project: &Project,
        table_name: &str,
    ) -> Result<()> {
        let db_name = project.get_dbname(self.get_name());
        log::debug!("removing table {} in database {}", table_name, db_name);
        self.get_user_db(project)
            .await?
            .remove_table(table_name)
            .await
    }
    /// Get table names from a user db
    pub async fn get_user_table_names(
        &mut self,
        project: &Project,
    ) -> Result<Vec<String>> {
        log::debug!("getting table names for project {}", project.name);
        self.get_user_db(project).await?.get_all_table_names().await
    }
    /// Get metadata on a user's table
    pub async fn get_user_table_meta(
        &mut self,
        project: &Project,
        table_name: &str,
    ) -> Result<TableMeta> {
        log::debug!(
            "getting table \"{}\" metadata in project \"{}\"",
            table_name,
            project.name
        );
        self.get_user_db(project)
            .await?
            .get_table_meta(table_name)
            .await
    }
    /// Get all tables metadata
    pub async fn get_all_meta(
        &mut self,
        project: &Project,
    ) -> Result<TableSpec> {
        log::debug!(
            "getting all metadata for project \"{}\"",
            project.get_name()
        );
        self.get_user_db(project).await?.get_all_meta().await
    }
    /// Insert data into a user's table
    pub async fn insert_user_table_data(
        &mut self,
        project: &Project,
        table_name: &str,
        data: &[RowJson],
    ) -> Result<()> {
        log::debug!(
            "inserting into table \"{}\" from project \"{}\"",
            table_name,
            project.name
        );
        self.get_user_db(project)
            .await?
            .insert_table_data(table_name, data)
            .await
    }
    /// Remove all data from a user's table
    pub async fn remove_all_user_table_data(
        &mut self,
        project: &Project,
        table_name: &str,
    ) -> Result<()> {
        log::debug!(
            "deleting all data from table \"{}\" in project \"{}\"",
            table_name,
            project.name
        );
        self.get_user_db(project)
            .await?
            .remove_all_table_data(table_name)
            .await
    }
    /// Get data from a user's table
    pub async fn get_user_table_data(
        &mut self,
        project: &Project,
        table_name: &str,
    ) -> Result<Vec<RowJson>> {
        log::debug!(
            "getting table \"{}\" metadata in project \"{}\"",
            table_name,
            project.name
        );
        self.get_user_db(project)
            .await?
            .get_table_data(table_name)
            .await
    }
}

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, sqlx::FromRow,
)]
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

#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, sqlx::FromRow,
)]
pub struct Project {
    user: i32,
    name: String,
    created: chrono::DateTime<chrono::Utc>,
}

impl Project {
    pub fn new(user: i32, name: &str) -> Self {
        Self {
            user,
            name: name.to_string(),
            created: chrono::Utc::now(),
        }
    }
    pub fn get_dbname(&self, admin_db_name: &str) -> String {
        format!("{}_user{}_{}", admin_db_name, self.user, self.name)
    }
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }
    pub fn get_user(&self) -> i32 {
        self.user
    }
    pub fn get_created(&self) -> chrono::DateTime<chrono::Utc> {
        self.created
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;

    const TEST_DB_NAME: &str = "odcadmin_test_admin";

    // Extract first admin
    async fn extract_first_user(db: &AdminDB) -> User {
        let user = sqlx::query_as::<Database, User>(
            "SELECT \"id\", \"email\", \"password_hash\", \"access\"\
            FROM \"user\" WHERE \"id\" = '1'",
        )
        .fetch_one(db.get_pool())
        .await
        .unwrap();
        log::info!("first user is {:?}", user);
        user
    }

    // Extract first admin's token
    async fn extract_first_user_token(db: &AdminDB) -> auth::Token {
        let token = sqlx::query_as::<Database, auth::Token>(
            "SELECT \"user\", \"token\", \"created\" FROM \"token\" \
            WHERE \"user\" = '1'",
        )
        .fetch_one(db.get_pool())
        .await
        .unwrap();
        log::info!("first user token is {:?}", token);
        token
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
        let rows = sqlx::query("SELECT datname FROM pg_database")
            .fetch_all(db.get_pool())
            .await
            .unwrap();
        let mut db_list = Vec::<String>::with_capacity(rows.len());
        for row in rows {
            db_list.push(row.get(0))
        }
        db_list
    }

    /// Verify that a project exists
    async fn project_exists(db: &AdminDB, project: &Project) -> bool {
        let db_exists = get_db_list(&db)
            .await
            .contains(&project.get_dbname(db.get_name()));
        let project_exists = db
            .get_all_projects()
            .await
            .unwrap()
            .iter()
            .any(|p| p.name == project.name && p.user == project.user);
        assert_eq!(db_exists, project_exists);
        db_exists
    }

    #[tokio::test]
    async fn test_admin() {
        let _ = pretty_env_logger::try_init();

        // Start clean
        log::info!("start clean");
        let test_db =
            crate::tests::create_test_admindb(TEST_DB_NAME, true, true).await;
        assert_eq!(test_db.get_name(), TEST_DB_NAME);

        // Token manipulation -------------------------------------------------

        log::info!("token manipulation");

        // Generate token
        let user1 = extract_first_user(&test_db).await;
        let tok1 = gen_tok(&test_db).await;
        let tok1_stored = extract_first_user_token(&test_db).await;
        assert_eq!(tok1.user(), tok1_stored.user());
        assert_eq!(tok1.created(), tok1_stored.created());
        assert_eq!(auth::hash_fast(tok1.token()), tok1_stored.token());

        // Restart
        log::info!("restart");
        test_db.get_pool().close().await;
        let test_db = crate::tests::create_test_admindb(
            "odcadmin_test_admin",
            false,
            false,
        )
        .await;
        // Data should not be modified
        let user2 = extract_first_user(&test_db).await;
        let tok2_stored = extract_first_user_token(&test_db).await;
        assert_eq!(user1, user2);
        assert_eq!(tok1_stored, tok2_stored);
        // Next generated token should be different
        let tok2_next = gen_tok(&test_db).await;
        assert_ne!(tok2_stored.token(), auth::hash_fast(tok2_next.token()));

        // Start clean again
        log::info!("start clean again");
        test_db.get_pool().close().await;
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
        let res =
            sqlx::query_as::<Database, auth::Token>("SELECT * FROM \"token\"")
                .fetch_all(test_db.get_pool())
                .await
                .unwrap();
        assert!(res.is_empty());

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

        // Refresh that token
        let user_tok_refreshed =
            test_db.refresh_token(user_tok.token()).await.unwrap();
        assert_eq!(user_tok.user(), user_tok_refreshed.user());
        assert!(user_tok.created() < user_tok_refreshed.created());
        assert_ne!(user_tok.token(), user_tok_refreshed.token());

        // Make that token appear older
        sqlx::query(
            "UPDATE \"token\" \
            SET \"created\" = '2000-08-14 08:15:29.425665+10' \
            WHERE \"user\" = '2'",
        )
        .execute(test_db.get_pool())
        .await
        .unwrap();
        let user = test_db.get_user_by_token(user_tok.token()).await;
        assert!(matches!(
            user,
            Err(Error::Unauthorized(Unauthorized::TokenTooOld))
        ));

        // Remove that token
        test_db.remove_token(user_tok.token()).await.unwrap();
        let user = test_db.get_user_by_token(user_tok.token()).await;
        assert!(matches!(
            user,
            Err(Error::Unauthorized(Unauthorized::NoSuchToken(_)))
        ));

        // User manipulation --------------------------------------------------

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

        log::info!("project manipulation");

        // Test projects
        let test_project1 = Project::new(1, "test");
        let test_project2 = Project::new(2, "test");
        let test_project1_dbname = test_project1.get_dbname(TEST_DB_NAME);
        let test_project2_dbname = test_project2.get_dbname(TEST_DB_NAME);

        assert_eq!(test_project1_dbname, "odcadmin_test_admin_user1_test");
        assert_eq!(test_project2_dbname, "odcadmin_test_admin_user2_test");

        // Make sure test projects aren't present
        log::info!("remove test projects");
        crate::tests::remove_dbs(
            &test_db,
            &[test_project1_dbname.as_str(), test_project2_dbname.as_str()],
        )
        .await;

        // Verify that the database does not exist
        assert!(!project_exists(&test_db, &test_project1).await);

        // Create project
        log::info!("create test project");
        test_db.create_project(1, "test").await.unwrap();

        log::info!("verify that database was created");
        assert!(project_exists(&test_db, &test_project1).await);
        let project = test_db.get_project(1, "test").await.unwrap();
        assert_eq!(project.name, test_project1.name);
        assert_eq!(project.user, test_project1.user);
        let err = test_db.get_project(1, "test1").await.unwrap_err();
        assert!(matches!(
                err,
                Error::NoSuchProject(id, name) if id == 1 && name == "test1"));

        log::info!("reconnect");
        test_db.get_pool().close().await;
        let test_db =
            crate::tests::create_test_admindb(TEST_DB_NAME, false, false).await;
        log::info!("verify that the project still exists");
        assert!(project_exists(&test_db, &test_project1).await);

        log::info!("reconnect cleanly");
        test_db.get_pool().close().await;
        let mut test_db = crate::tests::create_test_admindb(
            "odcadmin_test_admin",
            true,
            false,
        )
        .await;
        log::info!("verify that the project was removed");
        assert!(!project_exists(&test_db, &test_project1).await);

        log::info!("create the project again");
        test_db.create_project(1, "test").await.unwrap();
        assert!(project_exists(&test_db, &test_project1).await);

        log::info!("create the project as a different user");
        crate::tests::insert_test_user(&test_db).await;
        assert!(!project_exists(&test_db, &test_project2).await);
        test_db.create_project(2, "test").await.unwrap();
        assert!(project_exists(&test_db, &test_project2).await);
        assert_eq!(test_db.get_all_projects().await.unwrap().len(), 2);
        assert_eq!(test_db.get_user_projects(2).await.unwrap().len(), 1);

        // Get a project by name
        let user2_test_project =
            test_db.get_user_project(2, "test").await.unwrap();
        assert_eq!(user2_test_project.user, 2);
        assert_eq!(user2_test_project.name, "test");

        let nonexistent_project = test_db
            .get_user_project(2, "nonexistent")
            .await
            .unwrap_err();
        assert!(matches!(
            nonexistent_project,
            Error::NoSuchProject(id, name) if id == 2 && name == "nonexistent"
        ));

        log::info!("add a table to user project");
        let primary_table = crate::tests::get_test_primary_table();
        test_db
            .create_user_table(&user2_test_project, &primary_table)
            .await
            .unwrap();
        let user_db = test_db.get_user_db(&user2_test_project).await.unwrap();
        assert_eq!(
            user_db.get_all_table_names().await.unwrap(),
            vec![primary_table.name.clone()]
        );

        log::info!("remove that table");
        test_db
            .remove_user_table(&user2_test_project, primary_table.name.as_str())
            .await
            .unwrap();
        let user_db = test_db.get_user_db(&user2_test_project).await.unwrap();
        assert!(user_db.is_empty().await.unwrap());

        log::info!("remove all projects");
        test_db.remove_all_projects().await.unwrap();
        assert!(!project_exists(&test_db, &test_project2).await);
        assert!(!project_exists(&test_db, &test_project1).await);

        // Remove test db -----------------------------------------------------
        crate::tests::remove_test_db(&test_db).await;
    }
}
