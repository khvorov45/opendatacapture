use anyhow::{Context, Result};
use opendatacapture::{api, db, Opt};
use structopt::StructOpt;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    // Administrative database
    let admin_database =
        db::admin::AdminDB::new(db::admin::Config::from_opts(&opt))
            .await
            .context("failed to connect to administrative database")?;
    // API routes
    let routes = api::routes(std::sync::Arc::new(admin_database));
    // Start server
    warp::serve(routes).run(([127, 0, 0, 1], opt.apiport)).await;
    Ok(())
}
