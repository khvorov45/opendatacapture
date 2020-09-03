use crate::db::{ConnectionConfig, PoolMeta, DB};
use crate::Result;

pub mod table;

use table::TableMeta;

/// User project database
#[derive(Debug)]
pub struct UserDB {
    pool: PoolMeta,
}

#[async_trait::async_trait]
impl DB for UserDB {
    fn get_pool_meta(&self) -> &PoolMeta {
        &self.pool
    }
}

impl UserDB {
    pub async fn new(config: ConnectionConfig, name: &str) -> Result<Self> {
        Ok(Self {
            pool: PoolMeta::new(config, name).await?,
        })
    }
    /// Creates the given table
    pub async fn create_table(&self, table: &TableMeta) -> Result<()> {
        sqlx::query(table.construct_create_query().as_str())
            .execute(self.get_pool())
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_DB_NAME: &str = "odcadmin_test_user";

    // Test database
    #[tokio::test]
    async fn test_user() {
        let _ = pretty_env_logger::try_init();
        let test_config = crate::tests::gen_test_config("anything");
        crate::tests::setup_test_db(TEST_DB_NAME).await;
        let db = UserDB::new(test_config.clone(), TEST_DB_NAME)
            .await
            .unwrap();

        assert!(db.is_empty().await.unwrap());

        let primary_table = crate::tests::get_test_primary_table();

        db.create_table(&primary_table).await.unwrap();
        assert!(!db.is_empty().await.unwrap());
        assert!(db
            .get_all_table_names()
            .await
            .unwrap()
            .contains(&primary_table.name));

        let secondary_table = crate::tests::get_test_secondary_table();

        db.create_table(&secondary_table).await.unwrap();
        assert!(!db.is_empty().await.unwrap());
        assert!(db
            .get_all_table_names()
            .await
            .unwrap()
            .contains(&secondary_table.name));
    }
}
