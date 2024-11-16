use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use tracing::instrument;

use crate::file::FileInfo;

use super::error::HandlerError;

#[instrument]
pub async fn get_file_by_id(
    State(connection): State<PgPool>,
    Path(file_id): Path<i32>,
) -> Result<Bytes, HandlerError> {
    let file = FileInfo::get_file_by_id(&connection, file_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(file.into())
}

#[instrument]
pub async fn add_file(
    State(connection): State<PgPool>,
    payload: Bytes,
) -> Result<(), HandlerError> {
    FileInfo::insert_into_db(&connection, &payload)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

#[instrument]
pub async fn delete_file_by_id(
    State(connection): State<PgPool>,
    Path(file_id): Path<i32>,
) -> Result<(), HandlerError> {
    FileInfo::delete_from_db(&connection, file_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

#[instrument]
pub async fn get_all_files(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<FileInfo>>, HandlerError> {
    let files = FileInfo::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(files))
}

#[cfg(test)]

mod tests {
    use std::env;

    use axum::{
        body::{Body, Bytes},
        http::{Method, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use tower::{Service, ServiceExt}; // for `collect`

    use crate::{file::FileInfo, router::create_router};
    use sqlx::PgPool;
    use testcontainers::{ContainerAsync, ImageExt};
    use testcontainers_modules::{
        minio::{self, MinIO},
        postgres::{self, Postgres},
        testcontainers::runners::AsyncRunner,
    };

    async fn setup_database() -> (ContainerAsync<Postgres>, PgPool) {
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

    async fn setup_minio() -> ContainerAsync<MinIO> {
        let minio_container = minio::MinIO::default()
            .with_env_var("MINIO_ROOT_USER", "admin")
            .with_env_var("MINIO_ROOT_PASSWORD", "adminadmin")
            .start()
            .await
            .unwrap();
        let host_port = minio_container.get_host_port_ipv4(9000).await.unwrap();
        env::set_var("AWS_ACCESS_KEY_ID", "admin");
        env::set_var("AWS_SECRET_ACCESS_KEY", "adminadmin");
        env::set_var("AWS_REGION", "no");
        env::set_var("AWS_ENDPOINT", &format!("http://localhost:{}", host_port));
        minio_container
    }

    #[tokio::test]
    async fn should_add_file() {
        let (_container, pool) = setup_database().await;
        let _minio_container = setup_minio().await;
        let mut router = create_router(pool, None);

        let content = Bytes::from("Hello, world!");

        let add_file_request = Request::builder()
            .uri("/api/files")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(content.clone()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(add_file_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/files/1")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(get_request)
            .await
            .unwrap();
        dbg!(&response);
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body, content);
        let get_request = Request::builder()
            .uri("/api/file_infos")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(get_request)
            .await
            .unwrap();
        dbg!(&response);
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let file_infos = serde_json::from_slice::<Vec<FileInfo>>(&body).unwrap();
        assert_eq!(file_infos.len(), 1);
    }

    #[tokio::test]
    async fn should_remove_file() {
        let (_container, pool) = setup_database().await;
        let _minio_container = setup_minio().await;
        let mut router = create_router(pool, None);

        let content = Bytes::from("Hello, world!");

        let add_file_request = Request::builder()
            .uri("/api/files")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(content.clone()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(add_file_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let delete_request = Request::builder()
            .uri("/api/files/1")
            .method(Method::DELETE)
            .body(Body::empty())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(delete_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/file_infos")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(get_request)
            .await
            .unwrap();
        dbg!(&response);
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let file_infos = serde_json::from_slice::<Vec<FileInfo>>(&body).unwrap();
        assert_eq!(file_infos.len(), 0);
    }
}
