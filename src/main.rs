use tokio_postgres::{Error, NoTls};
use warp::Filter;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut dbconfig = tokio_postgres::config::Config::new();
    dbconfig
        .host("localhost")
        .user("odcdefault")
        .password("odcdefault")
        .port(5432);
    // Connect to the database.
    let (client, connection) = dbconfig.connect(NoTls).await?;

    // The connection object performs the actual communication with
    // the database,
    // so spawn it off to run on its own.
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
