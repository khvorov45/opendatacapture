use anyhow::{Context, Result};
use opendatacapture::{api, db, Opt};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::sync::Mutex;

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

    // Server parameters
    let addr = ([0, 0, 0, 0], opt.apiport);

    // Adding CORS changes type so hard I can't find a way to avoid
    // having to make both branches separate servers
    if opt.disable_cors {
        warp::serve(api::routes(admin_database_ref, opt.prefix.as_str()))
            .run(addr)
            .await;
    } else {
        warp::serve(api::routes_cors(admin_database_ref, opt.prefix.as_str()))
            .run(addr)
            .await;
    }

    Ok(())
}
