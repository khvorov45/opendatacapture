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
    /// Database host name
    #[structopt(long, default_value = "localhost")]
    pub dbhost: String,
    /// Database host port
    #[structopt(long, default_value = "5432")]
    pub dbport: u16,
    /// Admin database name.
    /// Will be used as an administrative database
    /// for keeping track of users.
    #[structopt(long, default_value = "odcadmin")]
    pub admindbname: String,
    /// API user name. Will be used to perform all database actions.
    #[structopt(long, default_value = "odcapi")]
    pub apiusername: String,
    /// API user password
    #[structopt(long, default_value = "odcapi")]
    pub apiuserpassword: String,
    /// Port for the api to listen to
    #[structopt(long, default_value = "4321")]
    pub apiport: u16,
    /// Do not backup and restore the data even if the admin database
    /// has tables.
    #[structopt(long)]
    pub clean: bool,
    /// Email for the first admin user
    #[structopt(long, default_value = "admin@example.com")]
    pub admin_email: String,
    /// Password for the first admin user
    #[structopt(long, default_value = "admin")]
    pub admin_password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test database config
    pub fn gen_test_config(dbname: &str) -> tokio_postgres::Config {
        let mut pg_config = tokio_postgres::Config::new();
        pg_config
            .host("localhost")
            .port(5432)
            .dbname(dbname)
            .user("odcapi")
            .password("odcapi");
        pg_config
    }

    /// Makes sure odcadmin_test database exists.
    /// Assumes odcadmin database exists
    pub async fn setup_test_db(dbname: &str) {
        let mut config = gen_test_config(dbname);
        config.dbname("odcadmin");
        let (odcadmin_client, con) =
            config.connect(tokio_postgres::NoTls).await.unwrap();
        tokio::spawn(async move {
            con.await.unwrap();
        });
        odcadmin_client
            .execute(
                format!("DROP DATABASE IF EXISTS {}", dbname).as_str(),
                &[],
            )
            .await
            .unwrap();
        // Drop test projects
        odcadmin_client
            .execute("DROP DATABASE IF EXISTS user1_test", &[])
            .await
            .unwrap();
        odcadmin_client
            .execute("DROP DATABASE IF EXISTS user1_test_api", &[])
            .await
            .unwrap();
        odcadmin_client
            .execute("DROP DATABASE IF EXISTS user2_test", &[])
            .await
            .unwrap();
        odcadmin_client
            .execute(format!("CREATE DATABASE {}", dbname).as_str(), &[])
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
        let pg_config = gen_test_config(dbname);
        let admin_conf = db::admin::Config {
            config: pg_config,
            clean,
            admin_email: "admin@example.com".to_string(),
            admin_password: "admin".to_string(),
        };
        db::admin::AdminDB::new(admin_conf).await.unwrap()
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
}
