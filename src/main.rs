use opendatacapture::{run, Opt};
use structopt::StructOpt;
use tokio_postgres::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    run(opt).await
}
