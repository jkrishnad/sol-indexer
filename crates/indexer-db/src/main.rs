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
    tracing_subscriber::fmt::init();
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
