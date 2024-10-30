use anyhow::Result;
use s3::{creds::Credentials, Bucket, BucketConfiguration, Region};
use serde::{Deserialize, Serialize};
use sha256::digest;
use sqlx::{prelude::FromRow, PgPool};

static BUCKET_NAME: &'static str = "files";

/// Generates a file name based on the id and hash.
fn file_name(id: i32, hash: &str) -> String {
    format!("{}-{}", id, hash)
}

/// Retrieves S3 credentials from the environment.
fn get_s3_credentials() -> Result<(Credentials, Region)> {
    Ok((Credentials::default()?, Region::from_default_env()?))
}

type Content = Vec<u8>;

/// Represents a file with its content.
#[derive(Debug)]
pub struct File {
    content: Content,
}

impl File {
    /// Creates a new [`File`].
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }

    /// Uploads the file to S3.
    pub async fn put_into_s3(
        &self,
        id: i32,
        hash: &str,
        credentials: Credentials,
        region: Region,
    ) -> Result<()> {
        let bucket =
            Bucket::new(BUCKET_NAME, region.clone(), credentials.clone())?.with_path_style();

        if !bucket.exists().await? {
            Bucket::create_with_path_style(
                BUCKET_NAME,
                region.clone(),
                credentials.clone(),
                BucketConfiguration::default(),
            )
            .await?;
        }

        bucket
            .put_object(file_name(id, hash), &self.content)
            .await?;

        Ok(())
    }

    /// Retrieves the file from S3.
    pub async fn get_from_s3(
        id: i32,
        hash: &str,
        credentials: Credentials,
        region: Region,
    ) -> Result<Self> {
        let bucket = Bucket::new(BUCKET_NAME, region.clone(), credentials.clone())
            .unwrap()
            .with_path_style();

        let result = bucket.get_object(file_name(id, hash)).await?;
        Ok(Self::new(result.into()))
    }

    /// Deletes the file from S3.
    pub async fn delete_from_s3(
        id: i32,
        hash: &str,
        credentials: Credentials,
        region: Region,
    ) -> Result<()> {
        let bucket = Bucket::new(BUCKET_NAME, region.clone(), credentials.clone())
            .unwrap()
            .with_path_style();

        bucket.delete_object(file_name(id, hash)).await?;

        Ok(())
    }
}

/// Represents file information stored in the database.
#[derive(FromRow, Serialize, Deserialize, Clone, Debug)]
pub struct FileInfo {
    id: i32,
    hash: String,
    object_storage_location: String,
}

impl FileInfo {
    /// Creates a new [`FileInfo`].
    pub fn new(id: i32, hash: String, object_storage_location: String) -> Self {
        Self {
            id,
            hash,
            object_storage_location,
        }
    }

    /// Inserts content into S3 and database.
    ///
    /// # Errors
    ///
    /// This function will return an error if S3 or DB is unavailable.
    pub async fn insert_into_db(pool: &PgPool, content: &[u8]) -> Result<()> {
        let hash = digest(content);
        let (credentials, region) = get_s3_credentials()?;
        let file = File::new(content.to_vec());
        sqlx::query("INSERT INTO files (hash, object_storage_location) VALUES ($1, $2)")
            .bind(hash.clone())
            .bind(BUCKET_NAME)
            .execute(pool)
            .await?;
        let id = sqlx::query_scalar("SELECT id FROM files WHERE hash = $1")
            .bind(hash.clone())
            .fetch_one(pool)
            .await?;
        file.put_into_s3(id, &hash, credentials, region).await?;
        Ok(())
    }

    /// Reads all file information from the database.
    pub async fn read_from_db(pool: &PgPool) -> Result<Vec<FileInfo>> {
        let files = sqlx::query_as::<_, FileInfo>("SELECT * FROM files")
            .fetch_all(pool)
            .await?;
        Ok(files)
    }

    /// Deletes file information from the database and S3.
    pub async fn delete_from_db(pool: &PgPool, id: i32) -> Result<()> {
        let file_info = Self::read_from_db_by_id(pool, id).await?;
        let (credentials, region) = get_s3_credentials()?;
        File::delete_from_s3(file_info.id, &file_info.hash, credentials, region).await?;
        sqlx::query("DELETE FROM files WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Reads file information by id from the database.
    pub async fn read_from_db_by_id(pool: &PgPool, id: i32) -> Result<FileInfo> {
        let file_info = sqlx::query_as::<_, FileInfo>("SELECT * FROM files WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;
        Ok(file_info)
    }

    /// Retrieves file content by id from the database and S3.
    pub async fn get_file_by_id(pool: &PgPool, id: i32) -> Result<Content> {
        let file_info = Self::read_from_db_by_id(pool, id).await?;
        let (credentials, region) = get_s3_credentials()?;
        let file = File::get_from_s3(file_info.id, &file_info.hash, credentials, region).await?;
        Ok(file.content)
    }

    /// Reads all file information from the database and retrieves the corresponding files from S3.
    pub async fn read_from_db_and_s3(pool: &PgPool) -> Result<Vec<(FileInfo, File)>> {
        let (credentials, region) = get_s3_credentials()?;
        let file_infos = sqlx::query_as::<_, FileInfo>("SELECT * FROM files")
            .fetch_all(pool)
            .await?;

        let mut result: Vec<(FileInfo, File)> = Vec::new();
        for file_info in file_infos {
            let file = File::get_from_s3(
                file_info.id,
                &file_info.hash,
                credentials.clone(),
                region.clone(),
            )
            .await?;
            result.push((file_info.clone(), file));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    pub async fn create_and_read_from_everything(pool: PgPool) {
        FileInfo::insert_into_db(&pool, &[1, 2, 3, 4, 5])
            .await
            .unwrap();

        let files = FileInfo::read_from_db(&pool).await;

        dbg!(&files);

        assert!(files.is_ok());
        let files = files.unwrap();
        let file = files.first().unwrap();

        assert_eq!(file.id, 1);

        let files = FileInfo::read_from_db_and_s3(&pool).await.unwrap();

        let (file_info, file) = files.first().unwrap();

        assert_eq!(file_info.id, 1);
        assert_eq!(file.content, &[1, 2, 3, 4, 5]);

        let (credentials, region) = get_s3_credentials().unwrap();

        File::delete_from_s3(file_info.id, &file_info.hash, credentials, region)
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

        let file = File::new([1, 2, 3, 4].to_vec());

        let res = file
            .put_into_s3(2, "hei", credentials.clone(), region.clone())
            .await;
        assert!(res.is_ok());

        let res = File::delete_from_s3(2, "hei", credentials, region).await;
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

        let file = File::new([1, 2, 3].to_vec());

        let res = file
            .put_into_s3(3, "hei", credentials.clone(), region.clone())
            .await;
        assert!(res.is_ok());

        let file = File::get_from_s3(3, "hei", credentials.clone(), region.clone())
            .await
            .unwrap();

        assert_eq!(file.content, &[1, 2, 3]);

        let res = File::delete_from_s3(3, "hei", credentials, region).await;
        assert!(res.is_ok());
    }
}
