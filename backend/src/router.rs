use axum::{
    body::Bytes,
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
    category::{Category, NewCategory},
    error::HandlerError,
    file::FileInfo,
    item::{Item, NewItem},
    location::{Location, NewLocation}
};

/// Middleware to log the request method and URI, and measure the time taken to handle the request.
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

/// Creates the router with all the routes and middleware.
pub fn create_router(connection: PgPool) -> Router {
    Router::new()
        .route("/status/health", get(status))
        .route("/api/items", get(get_all_items))
        .route("/api/items/:id", get(get_item_by_id))
        .route("/api/items", post(add_item))
        .route("/api/items/:id", delete(delete_item_by_id))
        .route("/api/items", put(update_item))
        .route("/api/locations", get(get_all_locations))
        .route("/api/locations/:id", get(get_location_by_id))
        .route("/api/locations", post(add_location))
        .route("/api/locations/:id", delete(delete_location_by_id))
        .route("/api/locations", put(update_location))
        .route("/api/categories", get(get_all_categories))
        .route("/api/categories/:id", get(get_category_by_id))
        .route("/api/categories", post(add_category))
        .route("/api/categories/:id", delete(delete_category_by_id))
        .route("/api/categories", put(update_category))
        .route("/api/files/:id", get(get_file_by_id))
        .route("/api/files", post(add_file))
        .route("/api/files/:id", delete(delete_file_by_id))
        .route("/api/file_infos", get(get_all_files))
        .with_state(connection)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(profile_endpoint)),
        )
}

/// Health check endpoint.
async fn status() -> (StatusCode, String) {
    (StatusCode::OK, "Healthy".to_string())
}

/// Retrieves all items from the database.
async fn get_all_items(State(connection): State<PgPool>) -> Result<Json<Vec<Item>>, HandlerError> {
    let items = Item::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(items))
}

/// Retrieves an item by its ID from the database.
async fn get_item_by_id(
    State(connection): State<PgPool>,
    Path(item_id): Path<i32>,
) -> Result<Json<Item>, HandlerError> {
    let item = Item::read_from_db_by_id(&connection, item_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(item))
}

/// Adds a new item to the database.
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

/// Deletes an item by its ID from the database.
async fn delete_item_by_id(
    State(connection): State<PgPool>,
    Path(item_id): Path<i32>,
) -> Result<(), HandlerError> {
    Item::delete_from_db(&connection, item_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Updates an item in the database.
async fn update_item(
    State(connection): State<PgPool>,
    Json(item): Json<Item>,
) -> Result<(), HandlerError> {
    Item::update_in_db(&connection, &item)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Retrieves all locations from the database.
async fn get_all_locations(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<Location>>, HandlerError> {
    let locations = Location::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(locations))
}

/// Retrieves a location by its ID from the database.
async fn get_location_by_id(
    State(connection): State<PgPool>,
    Path(location_id): Path<i32>,
) -> Result<Json<Location>, HandlerError> {
    let location = Location::read_from_db_by_id(&connection, location_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(location))
}

/// Adds a new location to the database.
async fn add_location(
    State(connection): State<PgPool>,
    Json(payload): Json<NewLocation>,
) -> Result<(), HandlerError> {
    Location::insert_into_db(&connection, &payload.name, &payload.description)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Deletes a location by its ID from the database.
async fn delete_location_by_id(
    State(connection): State<PgPool>,
    Path(location_id): Path<i32>,
) -> Result<(), HandlerError> {
    Location::delete_from_db(&connection, location_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Updates a location in the database.
async fn update_location(
    State(connection): State<PgPool>,
    Json(location): Json<Location>,
) -> Result<(), HandlerError> {
    Location::update_in_db(&connection, &location)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Retrieves all categories from the database.
async fn get_all_categories(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<Category>>, HandlerError> {
    let categories = Category::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(categories))
}

/// Retrieves a category by its ID from the database.
async fn get_category_by_id(
    State(connection): State<PgPool>,
    Path(category_id): Path<i32>,
) -> Result<Json<Category>, HandlerError> {
    let category = Category::read_from_db_by_id(&connection, category_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(category))
}

/// Adds a new category to the database.
async fn add_category(
    State(connection): State<PgPool>,
    Json(payload): Json<NewCategory>,
) -> Result<(), HandlerError> {
    Category::insert_into_db(&connection, &payload.name, &payload.description)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Deletes a category by its ID from the database.
async fn delete_category_by_id(
    State(connection): State<PgPool>,
    Path(category_id): Path<i32>,
) -> Result<(), HandlerError> {
    Category::delete_from_db(&connection, category_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Updates a category in the database.
async fn update_category(
    State(connection): State<PgPool>,
    Json(category): Json<Category>,
) -> Result<(), HandlerError> {
    Category::update_in_db(&connection, &category)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Retrieves a file by its ID from the database and S3.
async fn get_file_by_id(
    State(connection): State<PgPool>,
    Path(file_id): Path<i32>,
) -> Result<Bytes, HandlerError> {
    let file = FileInfo::get_file_by_id(&connection, file_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(file.into())
}

/// Adds a new file to the database and S3.
async fn add_file(State(connection): State<PgPool>, payload: Bytes) -> Result<(), HandlerError> {
    FileInfo::insert_into_db(&connection, &payload)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Deletes a file by its ID from the database and S3.
async fn delete_file_by_id(
    State(connection): State<PgPool>,
    Path(file_id): Path<i32>,
) -> Result<(), HandlerError> {
    FileInfo::delete_from_db(&connection, file_id)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(())
}

/// Retrieves all file information from the database.
async fn get_all_files(
    State(connection): State<PgPool>,
) -> Result<Json<Vec<FileInfo>>, HandlerError> {
    let files = FileInfo::read_from_db(&connection)
        .await
        .map_err(|e| HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(files))
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use crate::{
        category::{Category, NewCategory},
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

    #[sqlx::test]
    pub async fn add_category(pool: PgPool) {
        let router = create_router(pool);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3005").await.unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let client = reqwest::Client::new();

        let category = NewCategory::new(
            "Books".to_string(),
            "Item where words are stored".to_string(),
        );

        client
            .post("http://localhost:3005/api/categories")
            .json(&category)
            .send()
            .await
            .unwrap();

        let categories: Vec<Category> = client
            .get("http://localhost:3005/api/categories")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let category = categories.first().unwrap();

        assert_eq!(category.name, "Books".to_string());
        assert_eq!(
            category.description,
            "Item where words are stored".to_string()
        );

        handle.abort();
        assert!(handle.await.is_err());
    }

    #[sqlx::test]
    pub async fn get_category_by_id(pool: PgPool) {
        let router = create_router(pool);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3006").await.unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let client = reqwest::Client::new();

        let category = NewLocation::new(
            "Books".to_string(),
            "Item where words are stored".to_string(),
        );

        client
            .post("http://localhost:3006/api/categories")
            .json(&category)
            .send()
            .await
            .unwrap();

        let category: Category = client
            .get("http://localhost:3006/api/categories/1")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(category.name, "Books".to_string());
        assert_eq!(
            category.description,
            "Item where words are stored".to_string()
        );

        handle.abort();
        assert!(handle.await.is_err());
    }

    #[sqlx::test]
    pub async fn delete_category_by_id(pool: PgPool) {
        let router = create_router(pool);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3007").await.unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let client = reqwest::Client::new();

        let category = NewCategory::new(
            "Books".to_string(),
            "Item where words are stored".to_string(),
        );

        client
            .post("http://localhost:3007/api/categories")
            .json(&category)
            .send()
            .await
            .unwrap();

        let category: Category = client
            .get("http://localhost:3007/api/categories/1")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(category.name, "Books".to_string());
        assert_eq!(
            category.description,
            "Item where words are stored".to_string()
        );

        client
            .delete("http://localhost:3007/api/categories/1")
            .send()
            .await
            .unwrap();

        let categories: Vec<Category> = client
            .get("http://localhost:3007/api/categories")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(categories.iter().any(|category| category.id == 1), false);

        handle.abort();
        assert!(handle.await.is_err());
    }

    #[sqlx::test]
    pub async fn update_category(pool: PgPool) {
        let router = create_router(pool);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3008").await.unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        let client = reqwest::Client::new();

        let category = NewCategory::new(
            "Books".to_string(),
            "Item where words are stored".to_string(),
        );

        client
            .post("http://localhost:3008/api/categories")
            .json(&category)
            .send()
            .await
            .unwrap();

        let mut category: Category = client
            .get("http://localhost:3008/api/categories/1")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(category.name, "Books".to_string());
        assert_eq!(
            category.description,
            "Item where words are stored".to_string()
        );

        category.description = "Item where words is stored".to_string();

        client
            .put("http://localhost:3008/api/categories")
            .json(&category)
            .send()
            .await
            .unwrap();

        let category2: Category = client
            .get("http://localhost:3008/api/categories/1")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        assert_eq!(category2.name, "Books".to_string());
        assert_eq!(
            category2.description,
            "Item where words is stored".to_string()
        );

        handle.abort();
        assert!(handle.await.is_err());
    }
}
