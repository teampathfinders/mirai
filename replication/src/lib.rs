#![deny(
    clippy::expect_used,
    clippy::get_unwrap,
    clippy::if_then_some_else_none,
    clippy::impl_trait_in_params,
    clippy::let_underscore_untyped,
    clippy::missing_assert_message,
    clippy::mutex_atomic,
    clippy::undocumented_unsafe_blocks,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::str_to_string,
    clippy::clone_on_ref_ptr,
    clippy::nursery,
    clippy::default_trait_access,
    clippy::doc_link_with_quotes,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::implicit_clone,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::large_futures,
    clippy::large_types_passed_by_value,
    clippy::large_stack_arrays,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::match_bool,
    clippy::missing_fields_in_debug,
    clippy::missing_panics_doc,
    clippy::redundant_closure_for_method_calls,
    clippy::single_match_else,
    clippy::too_many_lines,
    clippy::trivially_copy_pass_by_ref,
    clippy::unused_self,
    clippy::unused_async
)]
#![allow(dead_code)]
#![allow(clippy::use_self)]

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
