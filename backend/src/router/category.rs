use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use tracing::instrument;

use crate::category::{Category, NewCategory};

use super::error::HandlerError;

#[instrument]
pub async fn get_all_categories(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<Category>>, HandlerError> {
    let categories = Category::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(categories))
}

#[instrument]
pub async fn get_category_by_id(
    State(connection): State<PgPool>,
    Path(category_id): Path<i32>,
) -> Result<Json<Category>, HandlerError> {
    let category = Category::read_from_db_by_id(&connection, category_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(category))
}

#[instrument]
pub async fn add_category(
    State(connection): State<PgPool>,
    Json(payload): Json<NewCategory>,
) -> Result<(), HandlerError> {
    Category::insert_into_db(&connection, &payload.name, &payload.description)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

#[instrument]
pub async fn delete_category_by_id(
    State(connection): State<PgPool>,
    Path(category_id): Path<i32>,
) -> Result<(), HandlerError> {
    Category::delete_from_db(&connection, category_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

#[instrument]
pub async fn update_category(
    State(connection): State<PgPool>,
    Json(category): Json<Category>,
) -> Result<(), HandlerError> {
    Category::update_in_db(&connection, &category)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}
