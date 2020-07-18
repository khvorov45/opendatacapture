use opendatacapture::{run, Opt};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    run(opt).await
}
