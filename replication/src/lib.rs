use anyhow::Context;
use fred::{
    clients::RedisClient,
    interfaces::{ClientLike, HashesInterface, StreamsInterface},
    types::{RedisConfig, RespVersion, Server, ServerConfig},
};
use proto::bedrock::{MovePlayer, TextData, TextMessage};
use util::{size_of_string, BinaryWrite};

#[cfg(test)]
mod test;

pub struct Replicator {
    client: RedisClient,
}

impl Replicator {
    pub async fn new(host: &str, port: u16) -> anyhow::Result<Self> {
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

        client.connect();
        client.wait_for_connect().await?;

        tracing::info!("Connected to database successfully");

        Ok(Self { client })
    }

    pub async fn save_session(&self, xuid: u64, name: &str) -> anyhow::Result<()> {
        self.client
            .hset(format!("user:{}", xuid), ("name", name))
            .await
            .context("Unable to cache player XUID")
    }

    pub async fn move_player(&self, _xuid: u64, _data: &MovePlayer) -> anyhow::Result<()> {
        todo!()

        // let mut buf = Vec::with_capacity(6 * 4 + 8);
        // buf.write_vecf(&data.translation)?;
        // buf.write_vecf(&data.rotation)?;
        // buf.write_u64_le(xuid)?;

        // self.client
        //     .publish("user:transform", buf.as_ref())
        //     .await
        //     .context("Unable to update player position")
    }

    pub async fn text_msg(&self, msg: &TextMessage<'_>) -> anyhow::Result<()> {
        if let TextData::Chat { message, source } = msg.data {
            let mut buf = Vec::with_capacity(8 + size_of_string(message) + size_of_string(source));
            buf.write_u64_le(msg.xuid)?;
            buf.write_str(source)?;
            buf.write_str(message)?;

            self.client
                .xadd(
                    "user:text",
                    false,
                    None,
                    "*",
                    vec![("xuid", msg.xuid.to_string().as_str()), ("name", source), ("body", message)],
                )
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
