use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

/// Category for grouping items
#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewCategory {
    pub name: String,
    pub description: String,
}

impl NewCategory {
    /// Creates a new [`NewCategory`].
    pub fn new(name: String, description: String) -> Self {
        Self { name, description }
    }
}

impl Category {
    /// Read all categories from the database
    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>("SELECT * FROM categories")
            .fetch_all(pool)
            .await?;
        Ok(categories)
    }

    /// Read category by id from the database
    pub async fn read_from_db_by_id(pool: &PgPool, id: i32) -> Result<Category> {
        let category = sqlx::query_as::<_, Category>("SELECT * FROM categories l WHERE l.id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(category)
    }

    /// Write category to database
    pub async fn insert_into_db(pool: &PgPool, name: &str, description: &str) -> Result<()> {
        sqlx::query("INSERT INTO categories (name, description) VALUES ($1, $2)")
            .bind(name)
            .bind(description)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Remove category from database
    pub async fn delete_from_db(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM categories l WHERE l.id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Update category in database
    pub async fn update_in_db(pool: &PgPool, category: &Category) -> Result<()> {
        sqlx::query("UPDATE categories SET name = $1, description = $2 WHERE id = $3")
            .bind(&category.name)
            .bind(&category.description)
            .bind(category.id)
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

        Category::insert_into_db(&pool, "Books", "Place to read words")
            .await
            .unwrap();

        let categories = Category::read_from_db(&pool).await;

        assert!(categories.is_ok());
        let categories = categories.unwrap();
        let category = categories.first().unwrap();

        assert_eq!(category.name, "Books".to_string());
        assert_eq!(category.description, "Place to read words".to_string());
    }

    #[tokio::test]
    pub async fn select_by_id() {
        let (_container, pool) = setup().await;

        Category::insert_into_db(&pool, "Books", "Place to read words")
            .await
            .unwrap();

        let categories = Category::read_from_db_by_id(&pool, 1).await;

        assert!(categories.is_ok());
        let category = categories.unwrap();

        assert_eq!(category.name, "Books".to_string());
        assert_eq!(category.description, "Place to read words".to_string());
    }

    #[tokio::test]
    pub async fn delete() {
        let (_container, pool) = setup().await;

        Category::insert_into_db(&pool, "Books", "Place to read words")
            .await
            .unwrap();

        let categories = Category::read_from_db_by_id(&pool, 1).await;

        assert!(categories.is_ok());
        let category = categories.unwrap();

        assert_eq!(category.name, "Books".to_string());
        assert_eq!(category.description, "Place to read words".to_string());

        let res = Category::delete_from_db(&pool, category.id).await;

        assert!(res.is_ok());

        let category = Category::read_from_db_by_id(&pool, 1).await;

        assert!(category.is_err());
    }

    #[tokio::test]
    pub async fn update() {
        let (_container, pool) = setup().await;

        Category::insert_into_db(&pool, "Books", "Place to read words")
            .await
            .unwrap();

        let categories = Category::read_from_db_by_id(&pool, 1).await;

        assert!(categories.is_ok());
        let mut category = categories.unwrap();

        assert_eq!(category.name, "Books".to_string());
        assert_eq!(category.description, "Place to read words".to_string());

        category.description = "Place where words with meaning are written".to_string();
        let res = Category::update_in_db(&pool, &category).await;

        assert!(res.is_ok());

        let category2 = Category::read_from_db_by_id(&pool, 1).await.unwrap();
        assert_eq!(category2.name, "Books".to_string());
        assert_eq!(
            category2.description,
            "Place where words with meaning are written".to_string()
        );
    }
}
