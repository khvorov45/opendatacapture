use std::sync::Arc;
use structopt::StructOpt;

pub mod api;
pub mod db;
pub mod error;
pub mod json;
mod password;

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

/// Extract opt to build a database configuration
fn admin_db_conf_from_opt(opt: &Opt) -> db::admin::Config {
    let mut config = tokio_postgres::config::Config::new();
    config
        .host(opt.dbhost.as_str())
        .port(opt.dbport)
        .dbname(opt.admindbname.as_str())
        .user(opt.apiusername.as_str())
        .password(opt.apiuserpassword.as_str());
    db::admin::Config {
        config,
        clean: opt.clean,
        admin_email: opt.admin_email.to_string(),
        admin_password: opt.admin_password.to_string(),
    }
}

/// Runs the API with the supplied options
pub async fn run(opt: Opt) -> Result<()> {
    // Administrative database
    let admin_database =
        db::admin::AdminDB::new(admin_db_conf_from_opt(&opt)).await?;
    let admin_database = Arc::new(admin_database);
    // API routes
    let authenticate_email_password =
        api::authenticate_email_password(admin_database);
    warp::serve(authenticate_email_password)
        .run(([127, 0, 0, 1], opt.apiport))
        .await;
    Ok(())
}

impl warp::reject::Reject for db::Error {}
