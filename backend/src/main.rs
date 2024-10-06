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

#[derive(Debug, Clone, StructOpt)]
pub struct Opts {
    #[structopt(short, long, default_value = "0.0.0.0:3000")]
    host: String,

    #[structopt(
        short,
        long,
        env = "DATABASE_URL",
        default_value = "postgres://postgres:example@server:5432/postgres"
    )]
    db_url: String,

    #[structopt(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::from_args();
    SimpleLogger::new()
        .with_level(log::LevelFilter::from_str(&opts.log_level)?)
        .init()?;

    info!("Connecting to DB at {}", opts.db_url);
    let connection = PgPool::connect(&opts.db_url).await.unwrap();

    let router = router::create_router(connection);
    let listener = tokio::net::TcpListener::bind(opts.host).await?;
    axum::serve(listener, router).await?;
    Ok(())
}
