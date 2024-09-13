use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct Gifter {
    id: i32,
    firstname: String,
    lastname: String,
    notes: String,
    date_added: DateTime<Utc>,
}

impl Gifter {
    pub fn new(
        id: i32,
        firstname: String,
        lastname: String,
        notes: String,
        date_added: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            firstname,
            lastname,
            notes,
            date_added,
        }
    }

    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<Gifter>> {
        let gifter = sqlx::query_as::<_, Gifter>("SELECT * FROM gifters")
            .fetch_all(pool)
            .await?;
        Ok(gifter)
    }

    pub async fn read_from_db_by_id(pool: &PgPool, id: i32) -> Result<Gifter> {
        let item = sqlx::query_as::<_, Gifter>("SELECT * FROM gifters i WHERE i.id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(item)
    }

    pub async fn insert_into_db(
        pool: &PgPool,
        firstname: &str,
        lastname: &str,
        notes: &str,
        date_added: DateTime<Utc>,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO gifters (firstname, lastname, notes, date_added) VALUES ($1, $2, $3, $4)",
        )
        .bind(firstname)
        .bind(lastname)
        .bind(notes)
        .bind(date_added)
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
        Gifter::insert_into_db(&pool, "Ola", "Normann", "Han er grei", now)
            .await
            .unwrap();

        let gifters = Gifter::read_from_db(&pool).await;

        assert!(gifters.is_ok());
        let gifters = gifters.unwrap();
        let gifter = gifters.first().unwrap();

        assert_eq!(gifter.firstname, "Ola".to_string());
        assert_eq!(gifter.lastname, "Normann".to_string());
        assert_eq!(gifter.notes, "Han er grei".to_string());
        assert!((gifter.date_added - now).num_seconds() < 1);
    }

    #[sqlx::test]
    pub async fn select_by_id(pool: PgPool) {
        let now = Utc::now();
        Gifter::insert_into_db(&pool, "Ola", "Normann", "Han er grei", now)
            .await
            .unwrap();

        let gifter = Gifter::read_from_db_by_id(&pool, 1).await;

        assert!(gifter.is_ok());
        let gifter = gifter.unwrap();

        assert_eq!(gifter.id, 1);
        assert_eq!(gifter.firstname, "Ola".to_string());
        assert_eq!(gifter.lastname, "Normann".to_string());
        assert_eq!(gifter.notes, "Han er grei".to_string());
        assert!((gifter.date_added - now).num_seconds() < 1);
    }
}
