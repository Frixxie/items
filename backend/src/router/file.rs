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
