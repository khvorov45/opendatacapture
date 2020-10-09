use sqlx::Row;

use crate::db::{ConnectionConfig, PoolMeta, DB};
use crate::{Error, Result};

pub mod table;

use table::{ColMeta, ColSpec, ForeignKey, RowJson, TableMeta, TableSpec};

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
    /// Checks that the table exists, returns Err if not
    async fn check_table_exists(&self, name: &str) -> Result<()> {
        if !self
            .get_all_table_names()
            .await?
            .contains(&name.to_string())
        {
            return Err(Error::NoSuchTable(name.to_string()));
        }
        Ok(())
    }
    /// Creates the given table
    pub async fn create_table(&self, table: &TableMeta) -> Result<()> {
        if self.get_all_table_names().await?.contains(&table.name) {
            return Err(Error::TableAlreadyExists(table.name.clone()));
        }
        sqlx::query(table.construct_create_query().as_str())
            .execute(self.get_pool())
            .await?;
        Ok(())
    }

    /// Removes a table
    pub async fn remove_table(&self, table_name: &str) -> Result<()> {
        log::debug!("removing table {}", table_name);

        self.check_table_exists(table_name).await?;

        sqlx::query(format!("DROP TABLE \"{}\"", table_name).as_str())
            .execute(self.get_pool())
            .await?;
        Ok(())
    }

    /// Get all table metadata
    pub async fn get_table_meta(&self, table_name: &str) -> Result<TableMeta> {
        log::debug!("get metadata for {}", table_name);

        self.check_table_exists(table_name).await?;

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

    /// Get all tables metadata
    pub async fn get_all_meta(&self) -> Result<TableSpec> {
        let table_names = self.get_all_table_names().await?;
        let mut table_spec = TableSpec::with_capacity(table_names.len());
        for table_name in table_names {
            table_spec.push(self.get_table_meta(table_name.as_str()).await?)
        }
        Ok(table_spec)
    }

    /// Insert data into a table
    pub async fn insert_table_data(
        &self,
        table_name: &str,
        data: &[RowJson],
    ) -> Result<()> {
        use serde_json::Value;
        let table = self.get_table_meta(table_name).await?;
        if data.is_empty() {
            return Err(Error::InsertEmptyData);
        }
        for row in data {
            // Only keep the columns that are not null
            let col_names: Vec<String> = row
                .iter()
                .filter_map(|(k, v)| {
                    if v.is_null() {
                        None
                    } else {
                        Some(k.to_string())
                    }
                })
                .collect();
            let query = table.construct_param_insert_query(&col_names)?;
            let mut row_query = sqlx::query(query.as_str());
            for col_name in &col_names {
                match &row[col_name] {
                    Value::Number(n) => row_query = row_query.bind(n.as_f64()),
                    Value::String(s) => row_query = row_query.bind(s.as_str()),
                    Value::Bool(b) => row_query = row_query.bind(b),
                    // Everything else is just a json
                    other => row_query = row_query.bind(other),
                }
            }
            row_query.execute(self.get_pool()).await?;
        }
        Ok(())
    }

    /// Remove all data from a table
    pub async fn remove_all_table_data(&self, table_name: &str) -> Result<()> {
        self.check_table_exists(table_name).await?;
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
        self.check_table_exists(table_name).await?;
        let res = sqlx::query(
            format!("SELECT ROW_TO_JSON(\"{0}\".*) FROM \"{0}\"", table_name)
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

        log::info!("create the same table again");

        assert!(matches!(
            db.create_table(&primary_table).await.unwrap_err(),
            Error::TableAlreadyExists(name) if name == primary_table.name
        ));

        log::info!("get table names");

        assert_eq!(
            db.get_all_table_names().await.unwrap(),
            vec![primary_table.name.clone(), secondary_table.name.clone()]
        );

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

        log::info!("get all metadata");

        let all_meta = db.get_all_meta().await.unwrap();
        assert_eq!(
            all_meta,
            vec![primary_table.clone(), secondary_table.clone()]
        );

        log::info!("get data");

        assert_eq!(
            db.get_table_data(primary_table.name.as_str())
                .await
                .unwrap(),
            vec![]
        );

        log::info!("insert data");

        let primary_data = crate::tests::get_primary_data();
        let secondary_data_partial = crate::tests::get_secondary_data_part();
        let secondary_data_null = crate::tests::get_secondary_data_null();
        let secondary_data = crate::tests::get_secondary_data();

        // Concatenate secondary data into what's expected to be in the table
        let secondary_data_partial_filled: Vec<RowJson> =
            secondary_data_partial
                .iter()
                .cloned()
                .map(|mut row| {
                    row.insert("sick".to_string(), serde_json::Value::Null);
                    row.insert("symptoms".to_string(), serde_json::Value::Null);
                    row.insert(
                        "locations".to_string(),
                        serde_json::Value::Null,
                    );
                    row
                })
                .collect();
        let mut secondary_data_full: Vec<RowJson> =
            secondary_data_partial_filled.clone();
        secondary_data_full.append(&mut secondary_data_null.clone());
        secondary_data_full.append(&mut secondary_data.clone());

        db.insert_table_data(primary_table.name.as_str(), &primary_data)
            .await
            .unwrap();

        db.insert_table_data(
            secondary_table.name.as_str(),
            &secondary_data_partial,
        )
        .await
        .unwrap();

        db.insert_table_data(
            secondary_table.name.as_str(),
            &secondary_data_null,
        )
        .await
        .unwrap();

        db.insert_table_data(secondary_table.name.as_str(), &secondary_data)
            .await
            .unwrap();

        log::info!("insert empty data");

        assert!(matches!(
            db.insert_table_data(primary_table.name.as_str(), &[])
                .await
                .unwrap_err(),
            Error::InsertEmptyData
        ));

        log::info!("get data");

        assert_eq!(
            db.get_table_data(primary_table.name.as_str())
                .await
                .unwrap(),
            primary_data
        );

        assert_eq!(
            db.get_table_data(secondary_table.name.as_str())
                .await
                .unwrap(),
            secondary_data_full,
        );

        log::info!("remove data");

        db.remove_all_table_data(secondary_table.name.as_str())
            .await
            .unwrap();

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

        log::info!("remove table");

        db.remove_table(secondary_table.name.as_str())
            .await
            .unwrap();

        log::info!("get table names");

        assert_eq!(
            db.get_all_table_names().await.unwrap(),
            vec![primary_table.name.clone()]
        );

        log::info!("create table again");

        db.create_table(&secondary_table).await.unwrap();

        log::info!("remove table that others reference");

        assert!(matches!(
            db.remove_table(primary_table.name.as_str()).await.unwrap_err(),
            Error::Sqlx(sqlx::Error::Database(e))
                if e.code().unwrap() == "2BP01"
        ));

        // Remove test DB -----------------------------------------------------
        crate::tests::remove_test_db(&db).await;
    }
}
