use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct Item {
    pub id: i32,
    name: String,
    description: String,
    date_origin: DateTime<Utc>,
    date_recieved: DateTime<Utc>,
}

impl Item {
    pub fn new(
        id: i32,
        name: String,
        description: String,
        date_origin: DateTime<Utc>,
        date_recieved: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            date_origin,
            date_recieved,
        }
    }

    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>("SELECT * FROM items")
            .fetch_all(pool)
            .await?;
        Ok(items)
    }

    pub async fn insert_into_db(
        pool: &PgPool,
        name: &str,
        description: &str,
        date_origin: DateTime<Utc>,
        date_recieved: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query("INSERT INTO items (name, description, date_origin, date_recieved) VALUES ($1, $2, $3, $4)").bind(name).bind(description).bind(date_origin).bind(date_recieved).execute(pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    

    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    pub async fn create(pool: PgPool) {
        let now = Utc::now();
        Item::insert_into_db(&pool, "Hei", "Test", now, now)
            .await
            .unwrap();

        let items = Item::read_from_db(&pool).await;

        assert!(items.is_ok());
        let items = items.unwrap();
        let item = items.first().unwrap();

        assert_eq!(item.name, "Hei".to_string());
        assert_eq!(item.description, "Test".to_string());
        assert!((item.date_origin - now).num_seconds() < 1);
        assert!((item.date_recieved - now).num_seconds() < 1);
    }
}
