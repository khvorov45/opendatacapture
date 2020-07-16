use structopt::StructOpt;

/// opendatacapture
#[derive(StructOpt, Debug)]
pub struct Opt {
    /// Host name
    #[structopt(long, default_value = "localhost")]
    pub host: String,
    /// Host port
    #[structopt(long, default_value = "5432")]
    pub port: i32,
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

pub fn run(opt: Opt) {
    println!("{:?}", opt)
}
