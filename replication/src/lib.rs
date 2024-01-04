use std::ops::Deref;
use std::sync::Mutex;
use anyhow::Context;
use fred::{clients::RedisClient, interfaces::{ClientLike, KeysInterface, HashesInterface, PubsubInterface}, types::{RedisConfig, RespVersion, ServerConfig, Server}};
use proto::bedrock::{MovePlayer, TextData, TextMessage};
use util::{BinaryWrite, MutableBuffer, size_of_string};

#[cfg(test)] mod test;

const TRANSFORM_KEY: &str = "player:transform";
const XUID_KEY: &str = "player:xuid";
const TEXT_KEY: &str = "player:chat";

pub struct Replicator {
    client: RedisClient
}

impl Replicator {
    pub async fn new() -> anyhow::Result<Self> {
        let client = RedisClient::new(
            RedisConfig {
                version: RespVersion::RESP3,
                server: ServerConfig::Centralized {
                    server: Server {
                        host: "replication".into(),
                        port: 6379
                    }
                },
                ..Default::default()
            },
            None, None, None
        );
        let _ = client.connect();
        let _ = client.wait_for_connect().await?;

        Ok(Self {
            client
        })
    }

    pub async fn save_session(&self, xuid: u64, name: &str) -> anyhow::Result<()> {
        self.client.hset(format!("player:{}", xuid), 
        ("username", name))
            .await
            .context("Unable to cache player XUID")
    }

    pub async fn move_player(&self, xuid: u64, data: &MovePlayer) -> anyhow::Result<()> {
        let mut buf = MutableBuffer::with_capacity(6 * 4 + 8);
        buf.write_vecf(&data.translation)?;
        buf.write_vecf(&data.rotation)?;
        buf.write_u64_le(xuid)?;

        self.client.publish(TRANSFORM_KEY, buf.as_ref())
            .await
            .context("Unable to update player position")
    }

    pub async fn text_msg(&self, msg: &TextMessage<'_>) -> anyhow::Result<()> {
        if let TextData::Chat {
            message, source
        } = msg.data {
            let mut buf = MutableBuffer::with_capacity(8 + size_of_string(message) + size_of_string(source));
            buf.write_u64_le(msg.xuid)?;
            buf.write_str(source)?;
            buf.write_str(message)?;

            self.client.publish(TEXT_KEY, buf.as_ref())
                .await
                .context("Unable to publish chat message")
        } else {
            todo!()
        }
    }
}

