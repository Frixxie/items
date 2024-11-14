use axum::{extract::State, http::StatusCode};
use metrics_exporter_prometheus::PrometheusHandle;
use tracing::{instrument, warn};

#[instrument]
pub async fn status() -> (StatusCode, String) {
    (StatusCode::OK, "Healthy".to_string())
}

#[instrument]
pub async fn metrics(State(handle): State<Option<PrometheusHandle>>) -> String {
    match handle {
        Some(h) => h.render(),
        None => {
            warn!("Prometheus metrics exporter is not configured");
            "".to_string()
        }
    }
}

#[cfg(test)]

mod tests {
    use axum::{body::Body, http::Request, http::StatusCode};
    use sqlx::PgPool;
    use testcontainers_modules::{postgres, testcontainers::runners::AsyncRunner};
    use tower::ServiceExt;

    use crate::router::create_router;

    async fn setup() -> PgPool {
        let postgres_container = postgres::Postgres::default().start().await.unwrap();
        let host_port = postgres_container.get_host_port_ipv4(5432).await.unwrap();
        let connection_string =
            &format!("postgres://postgres:postgres@127.0.0.1:{host_port}/postgres",);
        let connection = PgPool::connect(&connection_string).await.unwrap();
        sqlx::migrate!("./migrations")
            .run(&connection)
            .await
            .unwrap();
        connection
    }

    #[tokio::test]
    pub async fn should_be_healthy() {
        let connection = setup().await;
        let router = create_router(connection, None);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/status/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    pub async fn should_get_metrics() {
        let connection = setup().await;
        let router = create_router(connection, None);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
