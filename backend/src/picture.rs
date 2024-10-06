use anyhow::Result;
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};
use serde::{Deserialize, Serialize};
use sha256::digest;
use sqlx::{prelude::FromRow, PgPool};

pub type Picture = Vec<u8>;

#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct PictureInfo {
    id: i32,
    item_id: i32,
    description: String,
    hash: String,
    object_storage_location: String,
}

impl PictureInfo {
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

    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<PictureInfo>> {
        let items = sqlx::query_as::<_, PictureInfo>("SELECT * FROM pictures")
            .fetch_all(pool)
            .await?;
        Ok(items)
    }

    pub async fn read_from_db_and_s3(pool: &PgPool) -> Result<Vec<(PictureInfo, Picture)>> {
        let (credentials, region) = Self::get_s3_credentials()?;
        let picture_infos = sqlx::query_as::<_, PictureInfo>("SELECT * FROM pictures")
            .fetch_all(pool)
            .await?;

        let mut result: Vec<(PictureInfo, Picture)> = Vec::new();
        for picture_info in picture_infos {
            let picture = Self::get_from_s3(
                picture_info.item_id,
                &picture_info.hash,
                credentials.clone(),
                region.clone(),
            )
            .await?;
            result.push((picture_info.clone(), picture));
        }
        Ok(result)
    }

    fn into_bucket_name(item_id: i32) -> String {
        format!("item-{}", item_id)
    }

    fn get_s3_credentials() -> Result<(Credentials, Region)> {
        Ok((Credentials::default()?, Region::from_default_env()?))
    }

    pub async fn insert_into_db(
        pool: &PgPool,
        item_id: i32,
        description: &str,
        picture: &[u8],
    ) -> Result<()> {
        let hash = digest(picture);
        let (credentials, region) = Self::get_s3_credentials()?;
        Self::put_into_s3(item_id, &hash, picture, credentials, region).await?;
        sqlx::query("INSERT INTO pictures (item_id, description, hash, object_storage_location) VALUES ($1, $2, $3, $4)").bind(item_id).bind(description).bind(hash.clone()).bind(Self::into_bucket_name(item_id)).execute(pool).await?;
        Ok(())
    }

    pub async fn put_into_s3(
        item_id: i32,
        hash: &str,
        picture: &[u8],
        credentials: Credentials,
        region: Region,
    ) -> Result<()> {
        let bucket = Bucket::new(
            &Self::into_bucket_name(item_id),
            region.clone(),
            credentials.clone(),
        )?
        .with_path_style();

        if !bucket.exists().await? {
            Bucket::create_with_path_style(
                &Self::into_bucket_name(item_id),
                region.clone(),
                credentials.clone(),
                BucketConfiguration::default(),
            )
            .await?;
        }

        bucket.put_object(hash, picture).await?;

        Ok(())
    }

    pub async fn get_from_s3(
        item_id: i32,
        hash: &str,
        credentials: Credentials,
        region: Region,
    ) -> Result<Vec<u8>> {
        let bucket = Bucket::new(
            &Self::into_bucket_name(item_id),
            region.clone(),
            credentials.clone(),
        )
        .unwrap()
        .with_path_style();

        let result = bucket.get_object(hash).await?;
        Ok(result.into())
    }

    pub async fn delete_from_s3(
        item_id: i32,
        hash: &str,
        credentials: Credentials,
        region: Region,
    ) -> Result<()> {
        let bucket = Bucket::new(
            &Self::into_bucket_name(item_id),
            region.clone(),
            credentials.clone(),
        )
        .unwrap()
        .with_path_style();

        bucket.delete_object(hash).await?;

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
    pub async fn create_and_read_from_everything(pool: PgPool) {
        let now = Utc::now();
        Item::insert_into_db(&pool, "Stol", "Noe å sitte på", now)
            .await
            .unwrap();

        let items = Item::read_from_db(&pool).await;

        assert!(items.is_ok());
        let items = items.unwrap();
        let item = items.first().unwrap();
        PictureInfo::insert_into_db(&pool, item.id, "Bilde av stol", &[1, 2, 3, 4, 5])
            .await
            .unwrap();

        let pictures = PictureInfo::read_from_db(&pool).await;

        dbg!(&pictures);

        assert!(pictures.is_ok());
        let pictures = pictures.unwrap();
        let picture = pictures.first().unwrap();

        assert_eq!(picture.id, 1);
        assert_eq!(picture.description, "Bilde av stol");

        let pictures = PictureInfo::read_from_db_and_s3(&pool).await.unwrap();

        let (picture, content) = pictures.first().unwrap();

        assert_eq!(picture.id, 1);
        assert_eq!(picture.description, "Bilde av stol");
        assert_eq!(content, &[1, 2, 3, 4, 5]);

        let (credentials, region) = PictureInfo::get_s3_credentials().unwrap();

        PictureInfo::delete_from_s3(picture.id, &picture.hash, credentials, region)
            .await
            .unwrap();
    }

    #[tokio::test]
    pub async fn insert_and_delete_into_s3() {
        let credentials =
            Credentials::new(Some("admin"), Some("adminadmin"), None, None, None).unwrap();
        let region = Region::Custom {
            region: "no".to_owned(),
            endpoint: "http://localhost:9000".to_owned(),
        };

        let res =
            PictureInfo::put_into_s3(123, "hei", &[1, 2, 3], credentials.clone(), region.clone())
                .await;
        assert!(res.is_ok());

        let res = PictureInfo::delete_from_s3(123, "hei", credentials, region).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    pub async fn insert_get_and_delete_s3() {
        let credentials =
            Credentials::new(Some("admin"), Some("adminadmin"), None, None, None).unwrap();
        let region = Region::Custom {
            region: "no".to_owned(),
            endpoint: "http://localhost:9000".to_owned(),
        };

        let res =
            PictureInfo::put_into_s3(1234, "hei", &[1, 2, 3], credentials.clone(), region.clone())
                .await;
        assert!(res.is_ok());

        let picture = PictureInfo::get_from_s3(1234, "hei", credentials.clone(), region.clone())
            .await
            .unwrap();

        assert_eq!(picture, &[1, 2, 3]);

        let res = PictureInfo::delete_from_s3(1234, "hei", credentials, region).await;
        assert!(res.is_ok());
    }
}
