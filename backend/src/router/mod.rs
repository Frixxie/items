use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post, put},
    Router,
};
use category::{
    add_category, delete_category_by_id, get_all_categories, get_category_by_id, update_category,
};
use common::{metrics, status};
use file::{add_file, delete_file_by_id, get_all_files, get_file_by_id};
use item::{add_item, delete_item_by_id, get_all_items, get_item_by_id, update_item};
use location::{
    add_location, delete_location_by_id, get_all_locations, get_location_by_id, update_location,
};
use metrics::histogram;
use metrics_exporter_prometheus::PrometheusHandle;
use sqlx::PgPool;
use tokio::time::Instant;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;

mod category;
mod common;
mod file;
mod item;
mod location;

async fn profile_endpoint(request: Request, next: Next) -> Response {
    let method = request.method().clone().to_string().to_lowercase();
    let uri = request.uri().clone().path().replace("/", ".");

    info!("Handling {} at {}", method, uri);

    let now = Instant::now();

    let response = next.run(request).await;

    let elapsed = now.elapsed();

    histogram!(format!("{method}{uri}.handler")).record(elapsed);

    info!(
        "Finished handling {} at {}, used {} ms",
        method,
        uri,
        elapsed.as_millis()
    );
    response
}

pub fn create_router(connection: PgPool, metrics_handler: PrometheusHandle) -> Router {
    Router::new()
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
        .route("/metrics", get(metrics))
        .with_state(metrics_handler)
        .route("/status/health", get(status))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(profile_endpoint)),
        )
}
