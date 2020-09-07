use sqlx::Row;

use crate::db::{ConnectionConfig, PoolMeta, DB};
use crate::{Error, Result};

pub mod table;

use table::{ColMeta, ColSpec, ForeignKey, RowJson, TableMeta};

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

    /// Removes a table
    pub async fn remove_table(&self, table_name: &str) -> Result<()> {
        log::debug!("removing table {}", table_name);
        sqlx::query(table::construct_drop_query(table_name).as_str())
            .execute(self.get_pool())
            .await?;
        Ok(())
    }

    /// Get all table metadata
    pub async fn get_table_meta(&self, table_name: &str) -> Result<TableMeta> {
        log::debug!("get metadata for {}", table_name);

        let mut cols = ColSpec::new();

        // Non-constraint-related metadata
        let res = sqlx::query(
            r#"
        SELECT
            cols.column_name,
            cols.data_type,
            cols.is_nullable
        FROM
            information_schema.columns AS cols
        WHERE cols.table_name = $1
        "#,
        )
        .bind(table_name)
        .fetch_all(self.get_pool())
        .await?;

        for row in res {
            cols.push(
                ColMeta::new()
                    .name(row.get("column_name"))
                    .postgres_type(row.get("data_type"))
                    .not_null(row.get::<&str, &str>("is_nullable") == "NO"),
            );
        }

        // Constraint-related metadata
        let res = sqlx::query(
            r#"
        SELECT
            kcu.column_name,
            tc.constraint_type,
            ccu.table_name AS foreign_table_name,
            ccu.column_name AS foreign_column_name
        FROM
            information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            LEFT JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
                AND tc.constraint_type = 'FOREIGN KEY'
        WHERE kcu.table_name = $1
        "#,
        )
        .bind(table_name)
        .fetch_all(self.get_pool())
        .await?;

        for row in res {
            // Can't have a column appear in this query but not the other one,
            // so unwrap is ok here.
            let i = cols
                .iter()
                .position(|c| c.name == row.get::<&str, &str>("column_name"))
                .unwrap();
            match row.get("constraint_type") {
                "PRIMARY KEY" => {
                    cols[i].primary_key = true;
                }
                "FOREIGN KEY" => {
                    cols[i].foreign_key = Some(ForeignKey::new(
                        row.get("foreign_table_name"),
                        row.get("foreign_column_name"),
                    ));
                }
                "UNIQUE" => {
                    cols[i].unique = true;
                }
                // Do nothing with these for now
                "CHECK" => {}
                // There shouldn't be any other contraint types in Postgres
                s => panic!("unexpected contraint type: {}", s),
            }
        }

        Ok(TableMeta::new(table_name, cols))
    }

    /// Insert data into a table
    pub async fn insert_table_data(
        &self,
        table_name: &str,
        data: &[RowJson],
    ) -> Result<()> {
        use serde_json::Value;
        let col_names: Vec<String> =
            data[0].keys().map(|k| k.to_string()).collect();
        let query = table::construct_insert_query(table_name, &col_names);
        for row in data {
            let mut row_query = sqlx::query(query.as_str());
            for col_name in &col_names {
                match &row[col_name] {
                    Value::Number(n) => row_query = row_query.bind(n.as_f64()),
                    Value::String(s) => row_query = row_query.bind(s.as_str()),
                    other => {
                        return Err(Error::InsertFormatUnimplemented(
                            other.clone(),
                        ))
                    }
                }
            }
            row_query.execute(self.get_pool()).await?;
        }
        Ok(())
    }

    /// Remove all data from a table
    pub async fn remove_all_table_data(&self, table_name: &str) -> Result<()> {
        sqlx::query(format!("DELETE FROM \"{}\"", table_name).as_str())
            .execute(self.get_pool())
            .await?;
        Ok(())
    }

    /// Get all data from a table
    pub async fn get_table_data(
        &self,
        table_name: &str,
    ) -> Result<Vec<RowJson>> {
        let res = sqlx::query(
            format!("SELECT ROW_TO_JSON(\"{0}\") FROM \"{0}\"", table_name)
                .as_str(),
        )
        .fetch_all(self.get_pool())
        .await?;
        let mut rows = Vec::with_capacity(res.len());
        for row in res {
            match row.get::<serde_json::Value, usize>(0).as_object() {
                Some(o) => rows.push(o.clone()),
                None => return Err(Error::RowParse(row.get(0))),
            }
        }
        Ok(rows)
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

        log::info!("create table");

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

        log::info!("get metadata");

        let primary_meta = db
            .get_table_meta(primary_table.name.as_str())
            .await
            .unwrap();
        assert_eq!(primary_meta, primary_table);
        let secondary_meta = db
            .get_table_meta(secondary_table.name.as_str())
            .await
            .unwrap();
        assert_eq!(secondary_meta, secondary_table);

        log::info!("get data");

        assert_eq!(
            db.get_table_data(primary_table.name.as_str())
                .await
                .unwrap(),
            vec![]
        );

        log::info!("insert data");

        let primary_data = crate::tests::get_primary_data();

        db.insert_table_data(primary_table.name.as_str(), &primary_data)
            .await
            .unwrap();

        log::info!("get data");

        assert_eq!(
            db.get_table_data(primary_table.name.as_str())
                .await
                .unwrap(),
            primary_data
        );

        log::info!("remove data");

        db.remove_all_table_data(primary_table.name.as_str())
            .await
            .unwrap();

        log::info!("get data");

        assert_eq!(
            db.get_table_data(primary_table.name.as_str())
                .await
                .unwrap(),
            vec![]
        );

        // Remove test DB -----------------------------------------------------
        crate::tests::remove_test_db(&db).await;
    }
}
