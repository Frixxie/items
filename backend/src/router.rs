use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    Json, Router,
};
use log::info;
use sqlx::PgPool;
use tokio::time::Instant;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    error::HandlerError,
    item::{Item, NewItem},
    picture::Picture,
};

pub async fn profile_endpoint(request: Request, next: Next) -> Response {
    let method = request.method().clone().to_string();
    let uri = request.uri().clone();
    info!("Handling {} at {}", method, uri);

    let now = Instant::now();

    let response = next.run(request).await;

    let elapsed = now.elapsed();

    info!(
        "Finished handling {} at {}, used {} ms",
        method,
        uri,
        elapsed.as_millis()
    );
    response
}

pub fn create_router(connection: PgPool) -> Router {
    Router::new()
        .route("/status/health", get(status))
        .route("/api/items", get(get_all_items))
        .route("/api/items/:user_id", get(get_item_by_id))
        .route("/api/item", post(add_item))
        .route("/api/items/:user_id", delete(delete_item_by_id))
        .route("/api/item", put(update_item))
        .route("/api/pictures", get(get_all_pictures))
        .with_state(connection)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(profile_endpoint)),
        )
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

async fn get_item_by_id(
    State(connection): State<PgPool>,
    Path(item_id): Path<i32>,
) -> Result<Json<Item>, HandlerError> {
    let item = Item::read_from_db_by_id(&connection, item_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

async fn add_item(
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

async fn delete_item_by_id(
    State(connection): State<PgPool>,
    Path(item_id): Path<i32>,
) -> Result<(), HandlerError> {
    Item::delete_from_db(&connection, item_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

async fn update_item(
    State(connection): State<PgPool>,
    Json(item): Json<Item>,
) -> Result<(), HandlerError> {
    Item::update_in_db(&connection, &item)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

async fn get_all_pictures(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<Picture>>, HandlerError> {
    let pictures = Picture::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(pictures))
}
