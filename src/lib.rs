use std::sync::Arc;
use structopt::StructOpt;
use warp::Filter;

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

/// Runs the API with the supplied options
pub async fn run(opt: Opt) -> Result<()> {
    // Administrative database
    let admin_database = db::admin::AdminDB::new(&opt).await?;
    let admin_database = Arc::new(admin_database);
    // API routes
    let authenticate = warp::post()
        .and(warp::path("authenticate"))
        .and(warp::path("email-password"))
        .and(warp::body::json())
        .and_then(move |cred: db::admin::EmailPassword| {
            let admin_database = admin_database.clone();
            async move {
                let auth =
                    admin_database.authenticate_email_password(cred).await;
                match auth {
                    Ok(res) => Ok(res),
                    Err(_) => Err(warp::reject()),
                }
            }
        })
        .map(|b: bool| warp::reply::json(&b));
    warp::serve(authenticate)
        .run(([127, 0, 0, 1], opt.apiport))
        .await;
    Ok(())
}
