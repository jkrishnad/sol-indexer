use anyhow::Result;
use colored::{ColoredString, Colorize};
use futures::{SinkExt, StreamExt};
use indexer_core::{
    AccountUpdate, BlockUpdate, EntryUpdate, SlotUpdate, TransactionStatusUpdate, TransactionUpdate,
};
use redis_adapter::Publisher;
use serde::Serialize;
use serde_json::to_string;
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcClient};
use yellowstone_grpc_proto::geyser::{
    SubscribeRequest, SubscribeRequestPing, subscribe_update::UpdateOneof,
};

use crate::filter::Filters;

// Channels we publish to the redis
pub const CH_ACCOUNTS: &str = "accounts";
pub const CH_BLOCKS: &str = "blocks";
pub const CH_TRANSACTIONS: &str = "transactions";
pub const CH_SLOTS: &str = "slots";
pub const CH_ENTRIES: &str = "entries";
pub const CH_TRANSACTION_STATUS: &str = "transaction_status";

/// One colour per channel, so each update type is distinguishable at a glance
/// while watching the stream go past.
fn colorize(channel: &str, data: &str) -> ColoredString {
    match channel {
        CH_ACCOUNTS => data.green(),
        CH_TRANSACTIONS => data.blue(),
        CH_SLOTS => data.on_bright_yellow(),
        CH_BLOCKS => data.yellow(),
        CH_ENTRIES => data.cyan(),
        CH_TRANSACTION_STATUS => data.magenta(),
        _ => data.normal(),
    }
}

/// Serialize an update and hand it to the publisher.
async fn publish_update<P, T>(publisher: &P, channel: &str, update: &T)
where
    P: Publisher,
    T: Serialize,
{
    let data = match to_string(update) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(channel, error = %e, "failed to serialize update");
            return;
        }
    };

    // Colour-coded payload at INFO so the stream is readable while you watch it.
    // This serializes and colours every message on the hot path — demote to
    // debug! before running against real testnet volume.
    tracing::info!(channel, payload = %colorize(channel, &data), "publishing update");

    match publisher.publisher(channel, data.as_bytes()).await {
        Ok(()) => tracing::debug!(channel, bytes = data.len(), "published update"),
        Err(e) => tracing::error!(channel, error = %e, "failed to publish update"),
    }
}

pub async fn run_geyser<P: Publisher>(
    rpc_url: &str,
    x_token: Option<String>,
    filters: &Filters,
    publisher: P,
) -> Result<()> {
    tracing::info!("Getting subscribe request from filters");
    let request = filters.to_subscribe_request();

    let tls = rpc_url.starts_with("https://");
    tracing::info!(rpc_url, tls, "connecting to geyser");

    let mut builder = GeyserGrpcClient::build_from_shared(rpc_url.to_string())?.x_token(x_token)?;

    if tls {
        builder = builder.tls_config(ClientTlsConfig::new().with_native_roots())?;
    }

    // connect to the geyser
    let mut client = builder.connect().await?;

    // Subscribing gives back BOTH halves of the bidirectional stream:
    // - subscribe_tx — us -> server, for follow-up requests (pongs, filter changes)
    // - stream       — server -> us, the actual updates
    // Dropping the sink is what left us unable to answer pings before.
    let (mut subscribe_tx, mut stream) = client.subscribe_with_request(Some(request)).await?;

    // Ids we stamp on our pings; the server echoes them back in its pong.
    let mut ping_id: i32 = 0;

    // processing the stream
    while let Some(msg) = stream.next().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                tracing::error!(error = %e, "error receiving message from geyser");
                continue;
            }
        };

        match msg.update_oneof {
            // convert, and if the conversion fails log it and skip this one message.
            // A bad message must not take down the whole process.
            Some(UpdateOneof::Account(a)) => match AccountUpdate::try_from(a) {
                Ok(update) => publish_update(&publisher, CH_ACCOUNTS, &update.info).await,
                Err(e) => tracing::warn!(error = ?e, "skipping bad account update"),
            },
            Some(UpdateOneof::Transaction(tx)) => match TransactionUpdate::try_from(tx) {
                Ok(update) => publish_update(&publisher, CH_TRANSACTIONS, &update).await,
                Err(e) => tracing::warn!(error = ?e, "skipping bad transaction update"),
            },
            Some(UpdateOneof::Slot(s)) => match SlotUpdate::try_from(s) {
                Ok(update) => publish_update(&publisher, CH_SLOTS, &update).await,
                Err(e) => tracing::warn!(error = ?e, "skipping bad slot update"),
            },
            Some(UpdateOneof::Block(b)) => match BlockUpdate::try_from(b) {
                Ok(update) => publish_update(&publisher, CH_BLOCKS, &update).await,
                Err(e) => tracing::warn!(error = ?e, "skipping bad block update"),
            },
            Some(UpdateOneof::Entry(e)) => match EntryUpdate::try_from(e) {
                Ok(update) => publish_update(&publisher, CH_ENTRIES, &update).await,
                Err(e) => tracing::warn!(error = ?e, "skipping bad entry update"),
            },
            Some(UpdateOneof::TransactionStatus(ts)) => {
                match TransactionStatusUpdate::try_from(ts) {
                    Ok(update) => publish_update(&publisher, CH_TRANSACTION_STATUS, &update).await,
                    Err(e) => tracing::warn!(error = ?e, "skipping bad transaction status"),
                }
            }
            Some(UpdateOneof::Ping(_)) => {
                // The server pings to check we are still here. The reply is an
                // ordinary SubscribeRequest with only `ping` filled in every
                // other field stays at its default, which the server reads as
                // "no change to my subscription".
                ping_id = ping_id.wrapping_add(1);

                let pong = SubscribeRequest {
                    ping: Some(SubscribeRequestPing { id: ping_id }),
                    ..Default::default()
                };

                if let Err(e) = subscribe_tx.send(pong).await {
                    // If we cannot write to the sink the connection is already
                    // gone. Returning Err means the caller — and the exit code —
                    // find out.
                    return Err(anyhow::anyhow!("failed to answer ping: {e}"));
                }

                tracing::debug!(id = ping_id, "answered ping");
            }
            Some(UpdateOneof::Pong(pong)) => {
                tracing::debug!(id = pong.id, "pong received");
            }
            Some(UpdateOneof::BlockMeta(block_meta)) => {
                tracing::debug!(slot = block_meta.slot, "block meta received");
            }
            None => {
                tracing::warn!("received empty update from geyser");
            }
        }
    }

    // Falling out of the loop means the server closed the stream. That is not
    // success — returning Ok here is what made a dead indexer exit 0 and look
    // healthy to systemd. A reconnect loop replaces this in phase 2.
    Err(anyhow::anyhow!("geyser stream closed"))
}
