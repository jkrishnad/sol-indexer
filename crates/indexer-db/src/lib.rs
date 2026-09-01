use anyhow::Result;
use indexer_core::TransactionUpdate;
use models::NewTransaction;
use redis_adapter::{Consumer, Redis};
use store::Store;
use tracing::info;
pub mod models;
mod schema;
pub mod store;

pub async fn run_consumer(store: Store, redis_url: &str, channel: &str) -> Result<()> {
    let redis = Redis::new(redis_url)?;

    // Consume from the "transactions" channel
    let store_for_handler = store.clone();
    redis
        .consumer(channel, move |msg| {
            let chars: String = msg.chars().take(100).collect();
            info!("Received message from Redis: {}", chars);

            // deserialize the incoming message into TransactionUpdate from geyser
            let tx_update: TransactionUpdate = match serde_json::from_str(&msg) {
                Ok(tx) => {
                    info!("Successfully deserialized TransactionUpdate");
                    tx
                }
                Err(e) => {
                    info!("Failed to deserialize message: {}", e);
                    info!("Message content: {}", msg);
                    return Err(anyhow::anyhow!("Deserialization failed: {}", e));
                }
            };

            // extract transaction info if present
            if let Some(tx_info) = tx_update.transaction {
                info!(
                    "Transaction info found: slot={}, index={}",
                    tx_update.slot, tx_info.index
                );

                // extract meta if present
                if let Some(meta) = tx_info.meta {
                    info!(
                        "Meta found: fee={}, pre_balances={}",
                        meta.fee,
                        meta.pre_balances.len()
                    );

                    // convert to NewTransaction format by mapping the fields
                    let new_tx = NewTransaction::new(
                        tx_update.slot as i64,
                        tx_info.signature.clone(),
                        tx_info.is_vote,
                        tx_info.index as i32,
                        Some(meta.fee as i64),
                        meta.compute_units_consumed.map(|u| u as i64),
                        Some(
                            meta.pre_balances
                                .into_iter()
                                .map(|b| Some(b as i64))
                                .collect(),
                        ),
                        Some(
                            meta.post_balances
                                .into_iter()
                                .map(|b| Some(b as i64))
                                .collect(),
                        ),
                        Some(meta.log_messages.into_iter().map(|m| Some(m)).collect()),
                        Some(
                            meta.pre_token_balance
                                .into_iter()
                                .map(|_| Some(0i64))
                                .collect(),
                        ),
                        Some(
                            meta.post_token_balance
                                .into_iter()
                                .map(|_| Some(0i64))
                                .collect(),
                        ),
                    );

                    // insert the new transaction into the database
                    info!("Inserting transaction into database...");
                    match store_for_handler.insert_transaction(&[new_tx]) {
                        Ok(count) => info!("Successfully inserted {} transaction(s)", count),
                        Err(e) => {
                            info!("Failed to insert transaction: {}", e);
                            return Err(e);
                        }
                    }
                } else {
                    info!("No meta found for transaction");
                }
            } else {
                info!("No transaction info found");
            }

            Ok(())
        })
        .await?;

    // need to implement consumers for the other channels
    // as for my current rpc not getting their data i'll impl them later

    info!("Consumer stopped");
    Ok(())
}
