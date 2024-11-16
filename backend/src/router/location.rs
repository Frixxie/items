use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use tracing::instrument;

use crate::location::{Location, NewLocation};

use super::error::HandlerError;

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
        location::{Location, NewLocation},
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
    pub async fn should_insert_and_get_locations() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let location = NewLocation {
            name: "Stua".to_string(),
            description: "Stua er koselig".to_string(),
        };

        let create_request = Request::builder()
            .uri("/api/locations")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&location).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/locations")
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
        let locations = serde_json::from_slice::<Vec<Location>>(&body).unwrap();
        assert_eq!(locations.len(), 1);
    }

    #[tokio::test]
    pub async fn should_insert_and_get_location_by_id() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let location = NewLocation {
            name: "location".to_string(),
            description: "description".to_string(),
        };

        let create_request = Request::builder()
            .uri("/api/locations")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&location).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/locations/1")
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
        let location = serde_json::from_slice::<Location>(&body).unwrap();
        assert_eq!(location.id, 1);
    }

    #[tokio::test]
    pub async fn should_insert_and_update_location() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let location = NewLocation {
            name: "location".to_string(),
            description: "description".to_string(),
        };

        let create_request = Request::builder()
            .uri("/api/locations")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&location).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/locations")
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
        let mut locations = serde_json::from_slice::<Vec<Location>>(&body).unwrap();
        let location = locations.first_mut().unwrap();

        location.name = "new name".to_string();

        let update_request = Request::builder()
            .uri("/api/locations")
            .method(Method::PUT)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&location).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(update_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let get_request = Request::builder()
            .uri("/api/locations")
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
        let mut locations = serde_json::from_slice::<Vec<Location>>(&body).unwrap();
        let location = locations.first_mut().unwrap();
        assert_eq!(location.name, "new name");
    }

    #[tokio::test]
    pub async fn should_insert_and_delete_location() {
        let (_postgres_container, connection) = setup().await;
        let mut router = create_router(connection, None);

        let location = NewLocation {
            name: "location".to_string(),
            description: "description".to_string(),
        };

        let create_request = Request::builder()
            .uri("/api/locations")
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_string(&location).unwrap()))
            .unwrap();

        let response = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(create_request)
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let delete_request = Request::builder()
            .uri("/api/locations/1")
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
            .uri("/api/locations")
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
        let locations = serde_json::from_slice::<Vec<Location>>(&body).unwrap();
        assert_eq!(locations.len(), 0);
    }
}
