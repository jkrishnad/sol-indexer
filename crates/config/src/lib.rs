use once_cell::sync::Lazy;
use std::env;

pub struct Config {
    pub db_url: String,
    pub redis_url: String,
    pub rpc_url: String,
    pub x_token: Option<String>,
}

pub static CONFIG: Lazy<Config> = Lazy::new(|| {
    dotenv::dotenv().ok(); // Load .env only once
    Config {
        db_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        redis_url: env::var("REDIS_URL").expect("REDIS_URL must be set"),
        rpc_url: env::var("RPC_URL").expect("RPC_URL must be set"),
        x_token: env::var("RPC_API_KEY").ok(),
    }
});
