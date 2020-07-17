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
    /// Default database name
    #[structopt(long, default_value = "odcdefault")]
    pub dbname: String,
    /// User to connect to the database as
    #[structopt(long, default_value = "odcdefault")]
    pub dbuser: String,
    /// Database user password
    #[structopt(long, default_value = "odcdefault")]
    pub dbpassword: String,
    /// Port for the api to listen to
    #[structopt(long, default_value = "4321")]
    pub apiport: u16,
}

pub async fn run(opt: Opt) -> Result<(), Error> {
    let mut dbconfig = tokio_postgres::config::Config::new();
    dbconfig
        .host(opt.dbhost.as_str())
        .port(opt.dbport)
        .dbname(opt.dbname.as_str())
        .user(opt.dbuser.as_str())
        .password(opt.dbpassword);
    // Connect to the database.
    let (client, connection) = dbconfig.connect(NoTls).await?;
    // The connection object performs the actual communication with
    // the database, so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    // Now we can execute a simple statement that just returns its parameter.
    let rows = client.query("SELECT $1::TEXT", &[&"hello world"]).await?;
    println!("rows: {:?}", rows);
    let routes = warp::any().map(|| "Hello, World!");
    warp::serve(routes).run(([127, 0, 0, 1], opt.apiport)).await;
    Ok(())
}
