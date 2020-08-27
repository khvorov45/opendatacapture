use structopt::StructOpt;

pub mod api;
mod auth;
pub mod db;
pub mod error;
pub mod json;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// opendatacapture
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Database host name. Ignored if DATABASE_URL is defined.
    #[structopt(long, default_value = "localhost")]
    pub dbhost: String,
    /// Database host port. Ignored if DATABASE_URL is defined.
    #[structopt(long, default_value = "5432")]
    pub dbport: u16,
    /// Admin database name.
    /// Will be used as an administrative database for keeping track of users.
    /// Ignored if DATABASE_URL is defined.
    #[structopt(long, default_value = "odcadmin")]
    pub admindbname: String,
    /// API user name. Will be used to perform all database actions.
    /// Ignored if DATABASE_URL is defined.
    #[structopt(long, default_value = "odcapi")]
    pub apiusername: String,
    /// API user password.
    /// Ignored if DATABASE_URL is defined.
    #[structopt(long, default_value = "odcapi")]
    pub apiuserpassword: String,
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

    /// Test database config
    pub fn gen_test_config(dbname: &str) -> db::ConnectionConfig {
        db::ConnectionConfig::new()
            .host("localhost")
            .port(5432)
            .database(dbname)
            .username("odcapi")
            .password("odcapi")
    }

    /// Makes sure the test database database exists.
    /// Assumes the odcadmin database exists
    pub async fn setup_test_db(dbname: &str) {
        use sqlx::ConnectOptions;
        log::info!("setting up database {}", dbname);
        let config = gen_test_config(dbname).database("odcadmin");
        let mut con = config.connect().await.unwrap();
        sqlx::query(format!("DROP DATABASE IF EXISTS {0}", dbname).as_str())
            .execute(&mut con)
            .await
            .unwrap();
        sqlx::query(format!("CREATE DATABASE {0}", dbname).as_str())
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
        opt.admindbname = dbname.to_string();
        opt.clean = clean;
        db::admin::AdminDB::new(&opt).await.unwrap()
    }

    /// Insert a test user
    pub async fn insert_test_user(db: &db::admin::AdminDB) {
        db.insert_user(
            &db::admin::User::new(
                "user@example.com",
                "user",
                auth::Access::User,
            )
            .unwrap(),
        )
        .await
        .unwrap();
    }

    /// Remove specific databases
    pub async fn remove_dbs(db: &db::admin::AdminDB, names: &[&str]) {
        for name in names {
            db.execute(format!("DROP DATABASE IF EXISTS \"{}\"", name).as_str())
                .await
                .unwrap()
        }
    }
}
