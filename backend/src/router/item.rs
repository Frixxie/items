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
        .map_err(|e| {
            tracing::error!("Error: {}", e);
            HandlerError::new(StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
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

#[cfg(test)]

mod tests {
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use chrono::Utc;
    use http_body_util::BodyExt;
    use sqlx::PgPool;
    use testcontainers::ContainerAsync;
    use testcontainers_modules::{
        postgres::{self, Postgres},
        testcontainers::runners::AsyncRunner,
    };
    use tower::{Service, ServiceExt}; // for `collect`

    use crate::{
        item::{Item, NewItem},
        router::create_router,
    };

    async fn setup() -> (ContainerAsync<Postgres>, PgPool) {
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

    #[tokio::test]
    pub async fn should_insert_and_get_items() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let item = NewItem {
            name: "item".to_string(),
            description: "description".to_string(),
            date_origin: Utc::now(),
        };

        let create_request = Request::builder()
            .uri("/api/items")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&item).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/items")
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
        let items = serde_json::from_slice::<Vec<Item>>(&body).unwrap();
        assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    pub async fn should_insert_and_get_item_by_id() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let item = NewItem {
            name: "item".to_string(),
            description: "description".to_string(),
            date_origin: Utc::now(),
        };

        let create_request = Request::builder()
            .uri("/api/items")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&item).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/items/1")
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
        let item = serde_json::from_slice::<Item>(&body).unwrap();
        assert_eq!(item.id, 1);
    }

    #[tokio::test]
    pub async fn should_insert_and_update_item() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let item = NewItem {
            name: "item".to_string(),
            description: "description".to_string(),
            date_origin: Utc::now(),
        };

        let create_request = Request::builder()
            .uri("/api/items")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&item).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/items")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(get_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let mut items = serde_json::from_slice::<Vec<Item>>(&body).unwrap();
        let item = items.first_mut().unwrap();

        item.name = "new name".to_string();

        let update_request = Request::builder()
            .uri("/api/items")
            .method(Method::PUT)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&item).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(update_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/items")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(get_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let mut items = serde_json::from_slice::<Vec<Item>>(&body).unwrap();
        let item = items.first_mut().unwrap();
        assert_eq!(item.name, "new name");
    }

    #[tokio::test]
    pub async fn should_insert_and_delete_item() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let item = NewItem {
            name: "item".to_string(),
            description: "description".to_string(),
            date_origin: Utc::now(),
        };

        let create_request = Request::builder()
            .uri("/api/items")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&item).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let delete_request = Request::builder()
            .uri("/api/items/1")
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
            .uri("/api/items")
            .method(Method::GET)
            .body(Body::empty())
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(get_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let items = serde_json::from_slice::<Vec<Item>>(&body).unwrap();
        assert_eq!(items.len(), 0);
    }
}
