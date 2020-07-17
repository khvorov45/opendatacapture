use structopt::StructOpt;
use tokio_postgres::Error;
use warp::Filter;

mod admindb;

/// opendatacapture
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Database host name
    #[structopt(long, default_value = "localhost")]
    pub dbhost: String,
    /// Database host port
    #[structopt(long, default_value = "5432")]
    pub dbport: u16,
    /// Admin database name
    #[structopt(long, default_value = "odcadmin")]
    pub admindbname: String,
    /// Admin user name
    #[structopt(long, default_value = "odcadmin")]
    pub adminusername: String,
    /// Admin user password
    #[structopt(long, default_value = "odcadmin")]
    pub adminpassword: String,
    /// Port for the api to listen to
    #[structopt(long, default_value = "4321")]
    pub apiport: u16,
}

pub async fn run(opt: Opt) -> Result<(), Error> {
    pretty_env_logger::init();
    // Default database config as per the passed parameters
    let mut dbconfig = tokio_postgres::config::Config::new();
    dbconfig
        .host(opt.dbhost.as_str())
        .port(opt.dbport)
        .dbname(opt.admindbname.as_str())
        .user(opt.adminusername.as_str())
        .password(opt.adminpassword);
    // Connect to the admin database as the default admin user
    let admindb = admindb::AdminDB::connect(dbconfig).await?;
    // Now we can execute a simple statement that just returns its parameter.
    let rows = admindb
        .client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;
    println!("rows: {:?}", rows);
    let routes = warp::any().map(|| "Hello, World!");
    warp::serve(routes).run(([127, 0, 0, 1], opt.apiport)).await;
    Ok(())
}
