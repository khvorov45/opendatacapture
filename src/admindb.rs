use log::error;
use tokio_postgres::{Client, Error, NoTls};

pub struct AdminDB {
    pub client: Client,
}

impl AdminDB {
    pub async fn connect(
        config: tokio_postgres::config::Config,
    ) -> Result<Self, Error> {
        // Connect
        let (client, connection) = config.connect(NoTls).await?;
        // Spawn off the connection
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("connection error: {}", e);
            }
        });
        // Return the database object
        let admindb = Self { client };
        Ok(admindb)
    }
}
