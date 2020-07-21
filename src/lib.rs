use std::error::Error;
use structopt::StructOpt;
use warp::Filter;

mod db;
pub mod error;

/// opendatacapture
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Database host name
    #[structopt(long, default_value = "localhost")]
    pub dbhost: String,
    /// Database host port
    #[structopt(long, default_value = "5432")]
    pub dbport: u16,
    /// Admin database name.
    ///
    /// Will be used as an administrative database
    /// for keeping track of users.
    #[structopt(long, default_value = "odcadmin")]
    pub admindbname: String,
    /// API user name. Will be used to perform all database actions.
    #[structopt(long, default_value = "odcapi")]
    pub apiusername: String,
    /// API user password
    #[structopt(long, default_value = "odcapi")]
    pub apiuserpassword: String,
    /// Port for the api to listen to
    #[structopt(long, default_value = "4321")]
    pub apiport: u16,
    /// Force reset if database structure is incorrect
    #[structopt(long)]
    pub forcereset: bool,
    /// Force reset incorrect tables if found. Will cascade-drop (and re-create)
    /// all of those tables' dependencies.
    #[structopt(long)]
    pub forcetables: bool,
}

/// Runs the API with the supplied options
pub async fn run(opt: Opt) -> Result<(), Box<dyn Error>> {
    // Config
    let mut dbconfig = tokio_postgres::config::Config::new();
    dbconfig
        .host(opt.dbhost.as_str())
        .port(opt.dbport)
        .dbname(opt.admindbname.as_str())
        .user(opt.apiusername.as_str())
        .password(opt.apiuserpassword);
    // Connect to the admin database as the default api user
    let _admindb = db::DB::new(
        &dbconfig,
        get_admin_tablespec(),
        opt.forcereset,
        opt.forcetables,
    )
    .await?;
    let routes = warp::any().map(|| "Hello, World!");
    warp::serve(routes).run(([127, 0, 0, 1], opt.apiport)).await;
    Ok(())
}

fn get_admin_tablespec() -> db::TableSpec {
    let mut set = db::TableSpec::new();
    set.insert(String::from("admin"), get_admin_colspec());
    set
}

fn get_admin_colspec() -> db::ColSpec {
    let mut set = db::ColSpec::new();
    set.insert(
        String::from("id"),
        db::ColAttrib::new("SERIAL", "Primary Key"),
    );
    set.insert(
        String::from("email"),
        db::ColAttrib::new("TEXT", "NOT NULL"),
    );
    set
}
