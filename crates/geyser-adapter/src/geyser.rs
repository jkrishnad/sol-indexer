use anyhow::{Context, Result};
use colored::Colorize;
use indexer_core::{
    AccountUpdate, BlockUpdate, EntryUpdate, SlotUpdate, TransactionStatusUpdate, TransactionUpdate,
};
use futures::StreamExt;
use redis_adapter::Publisher;
use serde_json::to_string;
// use tonic::transport::Certificate;
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient};
use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;

use crate::filter::Filters;

// Channels we publish to the redis
pub const CH_ACCOUNTS: &str = "accounts";
pub const CH_BLOCKS: &str = "blocks";
pub const CH_TRANSACTIONS: &str = "transactions";
pub const CH_SLOTS: &str = "slots";
pub const CH_ENTRIES: &str = "entries";
pub const CH_TRANSACTION_STATUS: &str = "transaction_status";

pub async fn run_geyser<P: Publisher>(
    rpc_url: &str,
    x_token: Option<String>,
    filters: &Filters,
    publisher: P,
) -> Result<()> {
    println!("Getting subscribe request from filters");
    let request = filters.to_subscribe_request();

    println!("Connecting to geyser at {}", rpc_url);
    let tls_config = ClientTlsConfig::new().with_native_roots();

    // this is the optional CA cert loading from env var
    // if you have a custom rpc then you can set the CA_CERT env var to point to the cert file
    // for free rpc services like quicknode, alchemy, etc you don't need this
    // if let Some(ca_cert_path) = std::env::var_os("CA_CERT") {
    //     let bytes = tokio::fs::read(ca_cert_path).await?;
    //     tls_config = tls_config.ca_certificate(Certificate::from_pem(bytes));
    // }

    // create the connection
    let builder = GeyserGrpcClient::build_from_shared(rpc_url.to_string())?
        .x_token(x_token)?
        .tls_config(tls_config)?;

    // connect to the geyser
    let mut client = builder.connect().await?;

    // subscribe to the geyser
    let (_tx, mut stream) = client
        .subscribe_with_request(Some(request))
        .await
        .expect("Failed to subscribe to the geyser");

    // processing the stream
    while let Some(msg) = stream.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Error receiving message from geyser: {:?}", e);
                break;
            }
        };
        match msg.update_oneof {
            Some(UpdateOneof::Account(a)) => {
                let update = AccountUpdate::try_from(a)
                    .map_err(|e| anyhow::anyhow!(e))
                    .context("Failed to convert AccountUpdate")?;
                let data = to_string(&update.info)?;
                println!("Account data: {}", data.green());
                let _ = publisher.publisher(CH_ACCOUNTS, data.as_bytes()).await;
            }
            Some(UpdateOneof::Transaction(tx)) => {
                let update = TransactionUpdate::try_from(tx)
                    .map_err(|e| anyhow::anyhow!(e))
                    .context("Failed to convert TransactionUpdate")?;
                let data = to_string(&update)?;
                println!("Received Transaction update {}", data.blue());
                let _ = publisher.publisher(CH_TRANSACTIONS, data.as_bytes()).await;
            }
            Some(UpdateOneof::Slot(s)) => {
                let update = SlotUpdate::try_from(s)
                    .map_err(|e| anyhow::anyhow!(e))
                    .context("Failed to convert SlotUpdate")?;
                let data = to_string(&update)?;
                let _ = publisher.publisher(CH_SLOTS, data.as_bytes()).await;
            }
            Some(UpdateOneof::Block(b)) => {
                let update = BlockUpdate::try_from(b)
                    .map_err(|e| anyhow::anyhow!(e))
                    .context("Failed to convert BlockUpdate")?;
                let data = to_string(&update)?;
                println!("Received Block update {}", data.yellow());
                let _ = publisher.publisher(CH_BLOCKS, data.as_bytes()).await;
            }
            Some(UpdateOneof::Entry(e)) => {
                println!("Received Entry update {:?}", format!("{:?}", e).cyan());
                let update = EntryUpdate::try_from(e)
                    .map_err(|e| anyhow::anyhow!(e))
                    .context("Failed to convert EntryUpdate")?;
                let data = to_string(&update)?;
                let _ = publisher.publisher(CH_ENTRIES, data.as_bytes()).await;
            }
            Some(UpdateOneof::TransactionStatus(ts)) => {
                let update = TransactionStatusUpdate::try_from(ts)
                    .map_err(|e| anyhow::anyhow!(e))
                    .context("Failed to convert TransactionStatusUpdate")?;
                let data = to_string(&update)?;
                let _ = publisher.publisher(CH_TRANSACTION_STATUS, data.as_bytes()).await;
            }
            Some(UpdateOneof::Ping(p)) => {
                // Handle Ping update
                eprintln!("Received Ping update: {:?}", p);
            }
            Some(UpdateOneof::Pong(pong)) => {
                // Handle Pong update
                eprintln!("Received Pong update: {:?}", pong);
            }
            Some(UpdateOneof::BlockMeta(block_meta)) => {
                // Handle BlockMeta update
                eprintln!("Received BlockMeta update: {:?}", block_meta);
            }
            None => {
                eprintln!("Received empty update from geyser");
            }
        }
    }
    Ok(())
}
