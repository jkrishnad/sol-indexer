use anyhow::Result;
use futures::StreamExt;
use redis::Client;

// A trait for publishing messages to a channel
#[async_trait::async_trait]
pub trait Publisher: Send + Sync {
    async fn publisher(&self, channel: &str, payload: &[u8]) -> Result<(), anyhow::Error>;
}

#[async_trait::async_trait]
pub trait Consumer: Send + Sync {
    async fn consumer<F>(&self, channel: &str, handler: F) -> Result<(), anyhow::Error>
    where
        F: FnMut(String) -> Result<()> + Send + 'static;
}

pub struct Redis {
    pub client: Client,
}

impl Redis {
    pub fn new(rpc_url: &str) -> Result<Self> {
        let client = Client::open(rpc_url)?;
        Ok(Self { client })
    }
}

#[async_trait::async_trait]
impl Consumer for Redis {
    async fn consumer<F>(&self, channel: &str, mut handler: F) -> Result<(), anyhow::Error>
    where
        F: FnMut(String) -> Result<()> + Send + 'static,
    {
        let mut conn = self.client.get_async_pubsub().await?;
        conn.subscribe(channel).await?;
        let mut msg_stream = conn.on_message();

        while let Some(msg) = msg_stream.next().await {
            let payload: String = msg.get_payload()?;

            if let Err(e) = handler(payload) {
                eprintln!("Error handling message: {}", e);
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Publisher for Redis {
    async fn publisher(&self, channel: &str, payload: &[u8]) -> Result<(), anyhow::Error> {
        // getting a client connection
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        // converting the bytes which we get from the geyser into string
        let payload_str = String::from_utf8(payload.to_vec())?;

        // publishing the message which is the data to the respective channel
        redis::cmd("PUBLISH")
            .arg(channel)
            .arg(payload_str)
            .query_async::<()>(&mut connection).await?;

        Ok(())
    }
}
