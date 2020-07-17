use log::error;
use structopt::StructOpt;
use tokio_postgres::{Error, NoTls};
use warp::Filter;

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
    // Database config
    let mut dbconfig = tokio_postgres::config::Config::new();
    dbconfig
        .host(opt.dbhost.as_str())
        .port(opt.dbport)
        .dbname(opt.admindbname.as_str())
        .user(opt.adminusername.as_str())
        .password(opt.adminpassword);
    // Connect to the database.
    let (client, connection) = dbconfig.connect(NoTls).await?;
    // Spawn off the connection
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });
    // Now we can execute a simple statement that just returns its parameter.
    let rows = client.query("SELECT $1::TEXT", &[&"hello world"]).await?;
    println!("rows: {:?}", rows);
    let routes = warp::any().map(|| "Hello, World!");
    warp::serve(routes).run(([127, 0, 0, 1], opt.apiport)).await;
    Ok(())
}
