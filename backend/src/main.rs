mod category;
mod error;
mod file;
mod item;
mod location;
mod router;

use anyhow::Result;
use sqlx::PgPool;
use structopt::StructOpt;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(short, long, default_value = "0.0.0.0:3000")]
    host: String,

    #[structopt(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgresql://postgres:admin@localhost:5432/postgres"
    )]
    db_url: String,

    #[structopt(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::from_args();
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .json()
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    info!("Connecting to DB at {}", opts.db_url);
    let connection = PgPool::connect(&opts.db_url).await.unwrap();

    let router = router::create_router(connection);
    let listener = tokio::net::TcpListener::bind(opts.host).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
