use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct Location {
    id: i32,
    name: String,
    description: String,
}

impl Location {
    #[expect(dead_code)]
    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<Location>> {
        let locations = sqlx::query_as::<_, Location>("SELECT * FROM locations")
            .fetch_all(pool)
            .await?;
        Ok(locations)
    }

    #[expect(dead_code)]
    pub async fn read_from_db_by_id(pool: &PgPool, id: i32) -> Result<Location> {
        let location = sqlx::query_as::<_, Location>("SELECT * FROM locations l WHERE l.id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(location)
    }

    #[expect(dead_code)]
    pub async fn insert_into_db(pool: &PgPool, name: &str, description: &str) -> Result<()> {
        sqlx::query("INSERT INTO locations (name, description) VALUES ($1, $2)")
            .bind(name)
            .bind(description)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[expect(dead_code)]
    pub async fn delete_from_db(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM locations l WHERE l.id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    #[expect(dead_code)]
    pub async fn update_in_db(pool: &PgPool, location: &Location) -> Result<()> {
        sqlx::query("UPDATE locations SET name = $1, description = $2 WHERE id = $3")
            .bind(&location.name)
            .bind(&location.description)
            .bind(location.id)
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
        Location::insert_into_db(&pool, "Kitchen", "Where we make food")
            .await
            .unwrap();

        let locations = Location::read_from_db(&pool).await;

        assert!(locations.is_ok());
        let locations = locations.unwrap();
        let location = locations.first().unwrap();

        assert_eq!(location.name, "Kitchen".to_string());
        assert_eq!(location.description, "Where we make food".to_string());
    }

    #[sqlx::test]
    pub async fn select_by_id(pool: PgPool) {
        Location::insert_into_db(&pool, "Kitchen", "Where we make food")
            .await
            .unwrap();

        let locations = Location::read_from_db_by_id(&pool, 1).await;

        assert!(locations.is_ok());
        let location = locations.unwrap();

        assert_eq!(location.name, "Kitchen".to_string());
        assert_eq!(location.description, "Where we make food".to_string());
    }

    #[sqlx::test]
    pub async fn delete(pool: PgPool) {
        Location::insert_into_db(&pool, "Kitchen", "Where we make food")
            .await
            .unwrap();

        let locations = Location::read_from_db_by_id(&pool, 1).await;

        assert!(locations.is_ok());
        let location = locations.unwrap();

        assert_eq!(location.name, "Kitchen".to_string());
        assert_eq!(location.description, "Where we make food".to_string());

        let res = Location::delete_from_db(&pool, location.id).await;

        assert!(res.is_ok());

        let location = Location::read_from_db_by_id(&pool, 1).await;

        assert!(location.is_err());
    }

    #[sqlx::test]
    pub async fn update(pool: PgPool) {
        Location::insert_into_db(&pool, "Kitchen", "Where we make food")
            .await
            .unwrap();

        let locations = Location::read_from_db_by_id(&pool, 1).await;

        assert!(locations.is_ok());
        let mut location = locations.unwrap();

        assert_eq!(location.name, "Kitchen".to_string());
        assert_eq!(location.description, "Where we make food".to_string());

        location.description = "Where I make food".to_string();
        let res = Location::update_in_db(&pool, &location).await;

        assert!(res.is_ok());

        let location2 = Location::read_from_db_by_id(&pool, 1).await.unwrap();
        assert_eq!(location2.name, "Kitchen".to_string());
        assert_eq!(location2.description, "Where I make food".to_string());
    }
}
