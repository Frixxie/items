use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use sqlx::PgPool;

use crate::{error::HandlerError, gifter::Gifter, item::Item, picture::Picture};

pub fn create_router(connection: PgPool) -> Router {
    Router::new()
        .route("/status/health", get(status))
        .route("/api/items", get(get_all_items))
        .route("/api/pictures", get(get_all_pictures))
        .route("/api/gifters", get(get_all_gifters))
        .with_state(connection)
}

async fn status() -> (StatusCode, String) {
    (StatusCode::OK, "Healthy".to_string())
}

async fn get_all_items(State(connection): State<PgPool>) -> Result<Json<Vec<Item>>, HandlerError> {
    let items = Item::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(items))
}

async fn get_all_pictures(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<Picture>>, HandlerError> {
    let pictures = Picture::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(pictures))
}

async fn get_all_gifters(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<Gifter>>, HandlerError> {
    let gifters = Gifter::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(gifters))
}
