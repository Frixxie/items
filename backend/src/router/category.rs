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

#[cfg(test)]

mod tests {
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use http_body_util::BodyExt;
    use sqlx::PgPool;
    use testcontainers::ContainerAsync;
    use testcontainers_modules::{
        postgres::{self, Postgres},
        testcontainers::runners::AsyncRunner,
    };
    use tower::{Service, ServiceExt}; // for `collect`

    use crate::{
        category::{Category, NewCategory},
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
    pub async fn should_insert_and_get_categories() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let category = NewCategory {
            name: "Stue".to_string(),
            description: "hei".to_string(),
        };

        let create_request = Request::builder()
            .uri("/api/categories")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&category).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/categories")
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
        let categories = serde_json::from_slice::<Vec<Category>>(&body).unwrap();
        assert_eq!(categories.len(), 1);
    }

    #[tokio::test]
    pub async fn should_insert_and_get_category_by_id() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let category = NewCategory {
            name: "category".to_string(),
            description: "description".to_string(),
        };

        let create_request = Request::builder()
            .uri("/api/categories")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&category).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/categories/1")
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
        let category = serde_json::from_slice::<Category>(&body).unwrap();
        assert_eq!(category.id, 1);
    }

    #[tokio::test]
    pub async fn should_insert_and_update_category() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let category = NewCategory {
            name: "category".to_string(),
            description: "description".to_string(),
        };

        let create_request = Request::builder()
            .uri("/api/categories")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&category).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/categories")
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
        let mut categories = serde_json::from_slice::<Vec<Category>>(&body).unwrap();
        let category = categories.first_mut().unwrap();

        category.name = "new name".to_string();

        let update_request = Request::builder()
            .uri("/api/categories")
            .method(Method::PUT)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&category).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(update_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/categories")
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
        let mut categories = serde_json::from_slice::<Vec<Category>>(&body).unwrap();
        let category = categories.first_mut().unwrap();
        assert_eq!(category.name, "new name");
    }

    #[tokio::test]
    pub async fn should_insert_and_delete_category() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let category = NewCategory {
            name: "category".to_string(),
            description: "description".to_string(),
        };

        let create_request = Request::builder()
            .uri("/api/categories")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&category).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let delete_request = Request::builder()
            .uri("/api/categories/1")
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
            .uri("/api/categories")
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
        let categories = serde_json::from_slice::<Vec<Category>>(&body).unwrap();
        assert_eq!(categories.len(), 0);
    }
}
