use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha256::digest;
use sqlx::{prelude::FromRow, PgPool};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct Picture {
    id: i32,
    item_id: i32,
    description: String,
    hash: String,
    object_storage_location: String,
}

impl Picture {
    pub fn new(
        id: i32,
        item_id: i32,
        description: String,
        hash: String,
        object_storage_location: String,
    ) -> Self {
        Self {
            id,
            item_id,
            description,
            hash,
            object_storage_location,
        }
    }

    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<Picture>> {
        let items = sqlx::query_as::<_, Picture>("SELECT * FROM pictures")
            .fetch_all(pool)
            .await?;
        Ok(items)
    }

    pub async fn insert_into_db(
        pool: &PgPool,
        item_id: i32,
        description: &str,
        picture: &[u8],
    ) -> Result<()> {
        let hash = digest(picture);
        sqlx::query("INSERT INTO pictures (item_id, description, hash, object_storage_location) VALUES ($1, $2, $3, $4)").bind(item_id).bind(description).bind(hash).bind("Implement me!").execute(pool).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::item::Item;

    use super::*;
    use chrono::Utc;
    use sqlx::PgPool;

    #[sqlx::test]
    pub async fn create(pool: PgPool) {
        let now = Utc::now();
        Item::insert_into_db(&pool, "Hei", "Test", now)
            .await
            .unwrap();

        let items = Item::read_from_db(&pool).await;

        assert!(items.is_ok());
        let items = items.unwrap();
        let item = items.first().unwrap();
        Picture::insert_into_db(&pool, item.id, "Bilde av stol", &[1, 2, 3, 4, 5])
            .await
            .unwrap();

        let pictures = Picture::read_from_db(&pool).await;

        dbg!(&pictures);

        assert!(pictures.is_ok());
        let pictures = pictures.unwrap();
        let picture = pictures.first().unwrap();

        assert_eq!(picture.id, 1);
        assert_eq!(picture.description, "Bilde av stol")
    }
}
