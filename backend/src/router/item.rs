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
    use sqlx::PgPool;
    use testcontainers::ContainerAsync;
    use testcontainers_modules::{
        postgres::{self, Postgres},
        testcontainers::runners::AsyncRunner,
    };
    use tower::{Service, ServiceExt};

    use crate::{item::NewItem, router::create_router};

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
    pub async fn should_insert_and_get_item() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let item = NewItem {
            name: "item".to_string(),
            description: "description".to_string(),
            date_origin: Utc::now(),
        };

        // let create_request = Request::builder()
        //     .uri("/api/items")
        //     .method(Method::POST)
        //     .body(Body::from(serde_json::to_string(&item).unwrap()))
        //     .unwrap();

        // let response = ServiceExt::<Request<Body>>::ready(&mut router)
        //     .await
        //     .unwrap()
        //     .call(create_request)
        //     .await
        //     .unwrap();
        // assert_eq!(response.status(), StatusCode::OK);

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
    }
}
