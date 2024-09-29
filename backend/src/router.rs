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
    location::{Location, NewLocation},
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
        .route("/api/items", post(add_item))
        .route("/api/items/:user_id", delete(delete_item_by_id))
        .route("/api/items", put(update_item))
        .route("/api/locations", get(get_all_locations))
        .route("/api/locations/:user_id", get(get_location_by_id))
        .route("/api/locations", post(add_location))
        .route("/api/locations/:user_id", delete(delete_location_by_id))
        .route("/api/locations", put(update_location))
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

async fn get_all_locations(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<Location>>, HandlerError> {
    let locations = Location::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(locations))
}

async fn get_location_by_id(
    State(connection): State<PgPool>,
    Path(location_id): Path<i32>,
) -> Result<Json<Location>, HandlerError> {
    let location = Location::read_from_db_by_id(&connection, location_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(location))
}

async fn add_location(
    State(connection): State<PgPool>,
    Json(payload): Json<NewLocation>,
) -> Result<(), HandlerError> {
    Location::insert_into_db(&connection, &payload.name, &payload.description)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

async fn delete_location_by_id(
    State(connection): State<PgPool>,
    Path(location_id): Path<i32>,
) -> Result<(), HandlerError> {
    Location::delete_from_db(&connection, location_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

async fn update_location(
    State(connection): State<PgPool>,
    Json(location): Json<Location>,
) -> Result<(), HandlerError> {
    Location::update_in_db(&connection, &location)
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

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        location::{Location, NewLocation},
        router::create_router,
    };

    #[sqlx::test]
    pub async fn get_health(pool: PgPool) {
        let router = create_router(pool);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let client = reqwest::Client::new();

        let response = client
            .get("http://localhost:3000/status/health")
            .send()
            .await
            .unwrap();
        let body = response.text().await.unwrap();
        assert_eq!(body, "Healthy");

        handle.abort();
        assert!(handle.await.is_err());
    }

    #[sqlx::test]
    pub async fn add_location(pool: PgPool) {
        let router = create_router(pool);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let client = reqwest::Client::new();

        let location = NewLocation::new("Kitchen".to_string(), "Where we make food".to_string());

        client
            .post("http://localhost:3001/api/locations")
            .json(&location)
            .send()
            .await
            .unwrap();

        let locations: Vec<Location> = client
            .get("http://localhost:3001/api/locations")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let location = locations.first().unwrap();

        assert_eq!(location.name, "Kitchen".to_string());
        assert_eq!(location.description, "Where we make food".to_string());

        handle.abort();
        assert!(handle.await.is_err());
    }

    #[sqlx::test]
    pub async fn get_location_by_id(pool: PgPool) {
        let router = create_router(pool);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let client = reqwest::Client::new();

        let location = NewLocation::new("Kitchen".to_string(), "Where we make food".to_string());

        client
            .post("http://localhost:3002/api/locations")
            .json(&location)
            .send()
            .await
            .unwrap();

        let location: Location = client
            .get("http://localhost:3002/api/locations/1")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(location.name, "Kitchen".to_string());
        assert_eq!(location.description, "Where we make food".to_string());

        handle.abort();
        assert!(handle.await.is_err());
    }

    #[sqlx::test]
    pub async fn delete_location_by_id(pool: PgPool) {
        let router = create_router(pool);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3003").await.unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let client = reqwest::Client::new();

        let location = NewLocation::new("Kitchen".to_string(), "Where we make food".to_string());

        client
            .post("http://localhost:3003/api/locations")
            .json(&location)
            .send()
            .await
            .unwrap();

        let location: Location = client
            .get("http://localhost:3003/api/locations/1")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(location.name, "Kitchen".to_string());
        assert_eq!(location.description, "Where we make food".to_string());

        client
            .delete("http://localhost:3003/api/locations/1")
            .send()
            .await
            .unwrap();

        let locations: Vec<Location> = client
            .get("http://localhost:3003/api/locations")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(locations.iter().any(|location| location.id == 1), false);

        handle.abort();
        assert!(handle.await.is_err());
    }

    #[sqlx::test]
    pub async fn update_location(pool: PgPool) {
        let router = create_router(pool);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3004").await.unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let client = reqwest::Client::new();

        let location = NewLocation::new("Kitchen".to_string(), "Where we make food".to_string());

        client
            .post("http://localhost:3004/api/locations")
            .json(&location)
            .send()
            .await
            .unwrap();

        let mut location: Location = client
            .get("http://localhost:3004/api/locations/1")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(location.name, "Kitchen".to_string());
        assert_eq!(location.description, "Where we make food".to_string());

        location.description = "Where i make food".to_string();

        client
            .put("http://localhost:3004/api/locations")
            .json(&location)
            .send()
            .await
            .unwrap();

        let location2: Location = client
            .get("http://localhost:3004/api/locations/1")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(location2.name, "Kitchen".to_string());
        assert_eq!(location2.description, "Where i make food".to_string());

        handle.abort();
        assert!(handle.await.is_err());
    }
}
