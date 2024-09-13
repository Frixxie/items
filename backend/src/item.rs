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

#[derive(Deserialize, Clone, Debug)]
pub struct NewItem {
    pub name: String,
    pub description: String,
    pub date_origin: DateTime<Utc>,
    pub date_recieved: DateTime<Utc>,
}

impl Item {
    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<Item>> {
        let items = sqlx::query_as::<_, Item>("SELECT * FROM items")
            .fetch_all(pool)
            .await?;
        Ok(items)
    }

    pub async fn read_from_db_by_id(pool: &PgPool, id: i32) -> Result<Item> {
        let item = sqlx::query_as::<_, Item>("SELECT * FROM items i WHERE i.id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(item)
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

    pub async fn delete_from_db(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM items i WHERE i.id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update_in_db(pool: &PgPool, item: &Item) -> Result<()> {
        sqlx::query("UPDATE items SET name = $1, description = $2, date_origin = $3, date_recieved = $4 WHERE id = $5")
            .bind(&item.name).bind(&item.description).bind(item.date_origin).bind(item.date_recieved).bind(item.id)
            .execute(pool)
            .await?;
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

    #[sqlx::test]
    pub async fn select_by_id(pool: PgPool) {
        let now = Utc::now();
        Item::insert_into_db(&pool, "Hei", "Test", now, now)
            .await
            .unwrap();

        let item = Item::read_from_db_by_id(&pool, 1).await;

        assert!(item.is_ok());

        let item = item.unwrap();

        assert_eq!(item.id, 1);
        assert_eq!(item.name, "Hei".to_string());
        assert_eq!(item.description, "Test".to_string());
        assert!((item.date_origin - now).num_seconds() < 1);
        assert!((item.date_recieved - now).num_seconds() < 1);
    }

    #[sqlx::test]
    pub async fn delete(pool: PgPool) {
        let now = Utc::now();
        Item::insert_into_db(&pool, "Hei", "Test", now, now)
            .await
            .unwrap();

        let item = Item::read_from_db_by_id(&pool, 1).await;

        assert!(item.is_ok());

        let item = item.unwrap();

        assert_eq!(item.id, 1);
        assert_eq!(item.name, "Hei".to_string());
        assert_eq!(item.description, "Test".to_string());
        assert!((item.date_origin - now).num_seconds() < 1);
        assert!((item.date_recieved - now).num_seconds() < 1);

        let res = Item::delete_from_db(&pool, item.id).await;

        assert!(res.is_ok());

        let item = Item::read_from_db_by_id(&pool, 1).await;

        dbg!(&item);

        assert!(item.is_err());
    }

    #[sqlx::test]
    pub async fn update(pool: PgPool) {
        let now = Utc::now();
        Item::insert_into_db(&pool, "Hei", "Test", now, now)
            .await
            .unwrap();

        let item = Item::read_from_db_by_id(&pool, 1).await;

        assert!(item.is_ok());

        let mut item = item.unwrap();

        assert_eq!(item.id, 1);
        assert_eq!(item.name, "Hei".to_string());
        assert_eq!(item.description, "Test".to_string());
        assert!((item.date_origin - now).num_seconds() < 1);
        assert!((item.date_recieved - now).num_seconds() < 1);

        item.name = "Hallo".to_string();

        let res = Item::update_in_db(&pool, &item).await;

        dbg!(&res);

        assert!(res.is_ok());

        let item2 = Item::read_from_db_by_id(&pool, 1).await;

        assert!(item2.is_ok());

        let item2 = item2.unwrap();

        assert_eq!(item2.id, 1);
        assert_eq!(item2.name, "Hallo".to_string());
        assert_eq!(item2.description, "Test".to_string());
        assert!((item2.date_origin - now).num_seconds() < 1);
        assert!((item2.date_recieved - now).num_seconds() < 1);
    }
}
