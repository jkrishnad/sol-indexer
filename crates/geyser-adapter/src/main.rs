use anyhow::Result;
use filter::Filters;
use geyser::run_geyser;
use indexer_config::CONFIG;
use redis_adapter::Redis;
use rustls::crypto::{CryptoProvider, ring::default_provider};
mod filter;
mod geyser;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    CryptoProvider::install_default(default_provider()).ok();
    tracing::info!("Starting geyser adapter...");

    let rpc_url = &CONFIG.rpc_url;
    let redis_url = &CONFIG.redis_url;
    let x_token = &CONFIG.x_token;

    let filters_path = "filters.json";

    let filters = Filters::from_file(filters_path)?;

    let publisher = Redis::new(redis_url)?;
    run_geyser::<Redis>(rpc_url, x_token.clone(), &filters, publisher).await
}
