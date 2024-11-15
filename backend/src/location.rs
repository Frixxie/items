use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct Location {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewLocation {
    pub name: String,
    pub description: String,
}

impl NewLocation {
    /// Creates a new [`NewLocation`].
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl Location {
    /// Reads all locations from database
    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<Location>> {
        let locations = sqlx::query_as::<_, Location>("SELECT * FROM locations")
            .fetch_all(pool)
            .await?;
        Ok(locations)
    }

    /// Reads a location by id from database
    pub async fn read_from_db_by_id(pool: &PgPool, id: i32) -> Result<Location> {
        let location = sqlx::query_as::<_, Location>("SELECT * FROM locations l WHERE l.id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(location)
    }

    /// Insert location into database
    pub async fn insert_into_db(pool: &PgPool, name: &str, description: &str) -> Result<()> {
        sqlx::query("INSERT INTO locations (name, description) VALUES ($1, $2)")
            .bind(name)
            .bind(description)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Deletes a location from the database
    pub async fn delete_from_db(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM locations l WHERE l.id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Updates a location by id in the database
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
    use testcontainers::ContainerAsync;
    use testcontainers_modules::{
        postgres::{self, Postgres},
        testcontainers::runners::AsyncRunner,
    };

    async fn setup() -> (ContainerAsync<Postgres>, PgPool) {
        let postgres_container = postgres::Postgres::default().start().await.unwrap();
        let host_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
        let connection_string =
            &format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",);
        let connection = PgPool::connect(&connection_string).await.unwrap();
        sqlx::migrate!("./migrations")
            .run(&connection)
            .await
            .unwrap();
        (postgres_container, connection)
    }

    #[tokio::test]
    pub async fn create() {
        let (_container, pool) = setup().await;
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

    #[tokio::test]
    pub async fn select_by_id() {
        let (_container, pool) = setup().await;
        Location::insert_into_db(&pool, "Kitchen", "Where we make food")
            .await
            .unwrap();

        let locations = Location::read_from_db_by_id(&pool, 1).await;

        assert!(locations.is_ok());
        let location = locations.unwrap();

        assert_eq!(location.name, "Kitchen".to_string());
        assert_eq!(location.description, "Where we make food".to_string());
    }

    #[tokio::test]
    pub async fn delete() {
        let (_container, pool) = setup().await;
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

    #[tokio::test]
    pub async fn update() {
        let (_container, pool) = setup().await;
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
