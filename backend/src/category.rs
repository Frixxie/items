use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

/// Category for grouping items
#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct Category {
    id: i32,
    name: String,
    description: String,
}

impl Category {
    /// Read all categories from the database
    #[expect(dead_code)]
    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<Category>> {
        let categories = sqlx::query_as::<_, Category>("SELECT * FROM categories")
            .fetch_all(pool)
            .await?;
        Ok(categories)
    }

    /// Read category by id from the database
    #[expect(dead_code)]
    pub async fn read_from_db_by_id(pool: &PgPool, id: i32) -> Result<Category> {
        let category = sqlx::query_as::<_, Category>("SELECT * FROM categories l WHERE l.id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(category)
    }

    /// Write category to database
    #[expect(dead_code)]
    pub async fn insert_into_db(pool: &PgPool, name: &str, description: &str) -> Result<()> {
        sqlx::query("INSERT INTO categories (name, description) VALUES ($1, $2)")
            .bind(name)
            .bind(description)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Remove category from database
    #[expect(dead_code)]
    pub async fn delete_from_db(pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM categories l WHERE l.id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Update category in database
    #[expect(dead_code)]
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

    #[sqlx::test]
    pub async fn create(pool: PgPool) {
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

    #[sqlx::test]
    pub async fn select_by_id(pool: PgPool) {
        Category::insert_into_db(&pool, "Books", "Place to read words")
            .await
            .unwrap();

        let categories = Category::read_from_db_by_id(&pool, 1).await;

        assert!(categories.is_ok());
        let category = categories.unwrap();

        assert_eq!(category.name, "Books".to_string());
        assert_eq!(category.description, "Place to read words".to_string());
    }

    #[sqlx::test]
    pub async fn delete(pool: PgPool) {
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

    #[sqlx::test]
    pub async fn update(pool: PgPool) {
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
        assert_eq!(category2.description, "Place where words with meaning are written".to_string());
    }
}
