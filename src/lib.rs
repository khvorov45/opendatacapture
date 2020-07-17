use structopt::StructOpt;
use tokio_postgres::{Error, NoTls};
use warp::Filter;

/// opendatacapture
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Host name
    #[structopt(long, default_value = "localhost")]
    pub host: String,
    /// Host port
    #[structopt(long, default_value = "5432")]
    pub port: u16,
    /// Postgres database name to use as default
    #[structopt(long, default_value = "odcdefault")]
    pub dbname: String,
    /// User to connect ot Postgres as
    #[structopt(long, default_value = "odcdefault")]
    pub user: String,
    /// User password
    #[structopt(long, default_value = "odcdefault")]
    pub password: String,
}

pub async fn run(opt: Opt) -> Result<(), Error> {
    let mut dbconfig = tokio_postgres::config::Config::new();
    dbconfig
        .host(opt.host.as_str())
        .port(opt.port)
        .dbname(opt.dbname.as_str())
        .user(opt.user.as_str())
        .password(opt.password);
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
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}
