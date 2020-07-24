use std::error::Error;
use structopt::StructOpt;
use warp::Filter;

mod admindb;
pub mod db;
pub mod error;
mod password;

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
    /// for keeping track of users. Will be have its tables removed and
    /// re-created upon connection. Data will be backed up and restored
    /// unless the `--clean` option is passed.
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
pub async fn run(opt: Opt) -> Result<(), Box<dyn Error>> {
    // Administrative database
    let _admin_database = admindb::create_new(&opt).await?;
    // API routes
    let routes = warp::any().map(|| "Hello, World!");
    warp::serve(routes).run(([127, 0, 0, 1], opt.apiport)).await;
    Ok(())
}

/// Generates a random string
fn gen_rand_string() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(30)
        .collect()
}
