use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use tracing::instrument;

use crate::item::{Item, NewItem};

use super::error::HandlerError;

#[instrument]
pub async fn get_all_items(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<Item>>, HandlerError> {
    let items = Item::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(items))
}

#[instrument]
pub async fn get_item_by_id(
    State(connection): State<PgPool>,
    Path(item_id): Path<i32>,
) -> Result<Json<Item>, HandlerError> {
    let item = Item::read_from_db_by_id(&connection, item_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

#[instrument]
pub async fn add_item(
    State(connection): State<PgPool>,
    Json(payload): Json<NewItem>,
) -> Result<(), HandlerError> {
    Item::insert_into_db(
        &connection,
        &payload.name,
        &payload.description,
        payload.date_origin,
    )
    .await
    .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

#[instrument]
pub async fn delete_item_by_id(
    State(connection): State<PgPool>,
    Path(item_id): Path<i32>,
) -> Result<(), HandlerError> {
    Item::delete_from_db(&connection, item_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

#[instrument]
pub async fn update_item(
    State(connection): State<PgPool>,
    Json(item): Json<Item>,
) -> Result<(), HandlerError> {
    Item::update_in_db(&connection, &item)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}
