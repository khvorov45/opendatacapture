use std::sync::Arc;
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

/// Runs the API with the supplied options
pub async fn run(opt: Opt) -> Result<()> {
    // Administrative database
    let admin_database =
        db::admin::AdminDB::new(db::admin::Config::from_opts(&opt)).await?;
    // API routes
    let routes = api::routes(Arc::new(admin_database));
    warp::serve(routes).run(([127, 0, 0, 1], opt.apiport)).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test database config
    fn gen_test_config(dbname: &str) -> tokio_postgres::Config {
        let mut pg_config = tokio_postgres::Config::new();
        pg_config
            .host("localhost")
            .port(5432)
            .dbname(dbname)
            .user("odcapi")
            .password("odcapi");
        pg_config
    }

    // Makes sure odcadmin_test database exists.
    // Assumes odcadmin database exists
    async fn setup_test_db(dbname: &str) {
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
        odcadmin_client
            .execute(format!("CREATE DATABASE {}", dbname).as_str(), &[])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_lib() {
        let _ = pretty_env_logger::try_init();
        let dbname = "odcadmin_test_lib";
        setup_test_db(dbname).await;
        let opt = Opt::from_iter(["appname", "--admindbname", dbname].iter());
        tokio::spawn(async move {
            run(opt).await.unwrap();
        });
    }
}
