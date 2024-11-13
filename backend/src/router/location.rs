use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use tracing::instrument;

use crate::{
    error::HandlerError,
    location::{Location, NewLocation},
};

#[instrument]
pub async fn get_all_locations(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<Location>>, HandlerError> {
    let locations = Location::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(locations))
}

#[instrument]
pub async fn get_location_by_id(
    State(connection): State<PgPool>,
    Path(location_id): Path<i32>,
) -> Result<Json<Location>, HandlerError> {
    let location = Location::read_from_db_by_id(&connection, location_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(location))
}

#[instrument]
pub async fn add_location(
    State(connection): State<PgPool>,
    Json(payload): Json<NewLocation>,
) -> Result<(), HandlerError> {
    Location::insert_into_db(&connection, &payload.name, &payload.description)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

#[instrument]
pub async fn delete_location_by_id(
    State(connection): State<PgPool>,
    Path(location_id): Path<i32>,
) -> Result<(), HandlerError> {
    Location::delete_from_db(&connection, location_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

#[instrument]
pub async fn update_location(
    State(connection): State<PgPool>,
    Json(location): Json<Location>,
) -> Result<(), HandlerError> {
    Location::update_in_db(&connection, &location)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}
