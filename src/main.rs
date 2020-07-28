use anyhow::{Context, Result};
use opendatacapture::{run, Opt};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    run(opt).await.context("API error")
}
