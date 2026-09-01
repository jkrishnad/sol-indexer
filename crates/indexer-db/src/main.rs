use anyhow::Result;
use diesel::{
    pg::PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use indexer_config::CONFIG;
use indexer_db::{run_consumer, store::Store};
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // RUST_LOG wins when set; otherwise default to info. Without this,
    // EnvFilter::from_default_env() defaults to ERROR only and every info!
    // in the hot path is silently dropped — which looks exactly like a hang.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
    // get the required configurations
    let db_url = &CONFIG.db_url;
    let redis_url = &CONFIG.redis_url;

    info!("Starting DB consumer...");

    // create a database connection pool
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create database connection pool");

    // create store with the pool
    let store = Store::new(pool);

    // run consumer for transactions channel
    info!("Starting consumer for 'transactions' channel...");
    run_consumer(store, redis_url, "transactions").await?;

    Ok(())
}
