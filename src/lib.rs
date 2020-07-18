use std::error::Error;
use structopt::StructOpt;
use warp::Filter;

mod admindb;
mod error;

use admindb::DBState;
use error::APIError;

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
    ///
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
}

/// Runs the API with the supplied options
pub async fn run(opt: Opt) -> Result<(), Box<dyn Error>> {
    // Config
    let mut dbconfig = tokio_postgres::config::Config::new();
    dbconfig
        .host(opt.dbhost.as_str())
        .port(opt.dbport)
        .dbname(opt.admindbname.as_str())
        .user(opt.apiusername.as_str())
        .password(opt.apiuserpassword);
    // Connect to the admin database as the default api user
    let admindb = admindb::AdminDB::new(&dbconfig).await?;
    // The database can be in one of 3 states
    // Empty - need to init before use
    // Have the correct structure - no need to do anything
    // Have an incorrect structure - need to reset before use
    match admindb.state().await? {
        DBState::Empty => admindb.init().await?,
        DBState::Correct => (),
        DBState::Incorrect => {
            return Err(Box::new(APIError::new(
                "Admin database has incorrect structure, \
                    clear or reset it before use",
            )))
        }
    }
    let routes = warp::any().map(|| "Hello, World!");
    warp::serve(routes).run(([127, 0, 0, 1], opt.apiport)).await;
    Ok(())
}
