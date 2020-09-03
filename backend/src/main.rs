use anyhow::{Context, Result};
use opendatacapture::{api, db, Opt};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::Mutex;
use warp::Filter;

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    // Administrative database
    let admin_database = db::admin::AdminDB::new(&opt)
        .await
        .context("failed to connect to administrative database")?;
    let admin_database_ref = Arc::new(Mutex::new(admin_database));
    if opt.disable_cors {
        let routes = api::routes(admin_database_ref, opt.prefix.as_str());
        warp::serve(routes).run(([0, 0, 0, 0], opt.apiport)).await;
    } else {
        let routes = api::routes(admin_database_ref, opt.prefix.as_str())
            .with(api::get_cors());
        warp::serve(routes).run(([0, 0, 0, 0], opt.apiport)).await;
    }
    Ok(())
}
