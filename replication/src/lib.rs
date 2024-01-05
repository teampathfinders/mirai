use anyhow::Context;
use fred::{
    clients::RedisClient,
    interfaces::{ClientLike, HashesInterface, PubsubInterface, StreamsInterface},
    types::{RedisConfig, RespVersion, Server, ServerConfig, XCap},
};
use proto::bedrock::{MovePlayer, TextData, TextMessage};
use util::{size_of_string, BinaryWrite, MutableBuffer};

#[cfg(test)]
mod test;

pub struct Replicator {
    client: RedisClient,
}

impl Replicator {
    pub async fn new() -> anyhow::Result<Self> {
        let host = std::env::vars().find_map(|(k, v)| if k == "REDIS_HOST" { Some(v) } else { None });

        let host = if let Some(host) = host {
            host
        } else {
            tracing::debug!("No REDIS_HOST environment variable found, using default host 127.0.0.1");
            String::from("127.0.0.1")
        };

        let port = std::env::vars().find_map(|(k, v)| if k == "REDIS_PORT" { Some(v) } else { None });

        let port: u16 = if let Some(port) = port {
            port.parse().context("Failed to parse REDIS_PORT argument")?
        } else {
            tracing::debug!("No REDIS_PORT environment variable found, using default port 6379");
            6379
        };

        let client = RedisClient::new(
            RedisConfig {
                version: RespVersion::RESP3,
                server: ServerConfig::Centralized {
                    server: Server { host: host.into(), port },
                },
                ..Default::default()
            },
            None,
            None,
            None,
        );
        let _ = client.connect();
        client.wait_for_connect().await?;

        tracing::debug!("Replication layer created");

        Ok(Self { client })
    }

    pub async fn save_session(&self, xuid: u64, name: &str) -> anyhow::Result<()> {
        self.client
            .hset(format!("user:{}", xuid), ("name", name))
            .await
            .context("Unable to cache player XUID")
    }

    pub async fn move_player(&self, xuid: u64, data: &MovePlayer) -> anyhow::Result<()> {
        let mut buf = MutableBuffer::with_capacity(6 * 4 + 8);
        buf.write_vecf(&data.translation)?;
        buf.write_vecf(&data.rotation)?;
        buf.write_u64_le(xuid)?;

        self.client
            .publish("user:transform", buf.as_ref())
            .await
            .context("Unable to update player position")
    }

    pub async fn text_msg(&self, msg: &TextMessage<'_>) -> anyhow::Result<()> {
        if let TextData::Chat { message, source } = msg.data {
            let mut buf = MutableBuffer::with_capacity(8 + size_of_string(message) + size_of_string(source));
            buf.write_u64_le(msg.xuid)?;
            buf.write_str(source)?;
            buf.write_str(message)?;
            
            self.client
                .xadd("user:text", false, None, "*", vec![("xuid", msg.xuid.to_string().as_str()), ("name", source), ("body", message)])
                .await
                .context("Unable to add to user text stream")

            // self.client
            //     .publish("user:text", buf.as_ref())
            //     .await
            //     .context("Unable to publish chat message")
        } else {
            todo!()
        }
    }
}
