use axum::{extract::State, http::StatusCode};
use metrics_exporter_prometheus::PrometheusHandle;
use tracing::instrument;

#[instrument]
pub async fn status() -> (StatusCode, String) {
    (StatusCode::OK, "Healthy".to_string())
}

#[instrument]
pub async fn metrics(State(handle): State<PrometheusHandle>) -> String {
    handle.render()
}
