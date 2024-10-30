mod error;
mod item;
mod location;
mod file;
mod category;
mod router;

use std::str::FromStr;

use anyhow::Result;
use log::info;
use simple_logger::SimpleLogger;
use sqlx::PgPool;
use structopt::StructOpt;

/// Command line options for the application.
#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    /// Host address to bind the server to.
    #[structopt(short, long, default_value = "0.0.0.0:3000")]
    host: String,

    /// Database URL for connecting to the PostgreSQL database.
    #[structopt(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgresql://postgres:admin@localhost:5432/postgres"
    )]
    db_url: String,

    /// Log level for the application.
    #[structopt(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line options.
    let opts = Opts::from_args();

    // Initialize the logger with the specified log level.
    SimpleLogger::new()
        .with_level(log::LevelFilter::from_str(&opts.log_level)?)
        .init()?;

    info!("Connecting to DB at {}", opts.db_url);

    // Connect to the PostgreSQL database.
    let connection = PgPool::connect(&opts.db_url).await.unwrap();

    // Create the router with the database connection.
    let router = router::create_router(connection);

    // Bind the server to the specified host address.
    let listener = tokio::net::TcpListener::bind(opts.host).await?;

    // Start the server.
    axum::serve(listener, router).await?;
    Ok(())
}
