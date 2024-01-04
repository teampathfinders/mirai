use std::ops::Deref;
use std::sync::Mutex;
use redis::{AsyncCommands, ErrorKind, FromRedisValue, RedisError, RedisResult, RedisWrite, ToRedisArgs, Value};
use redis::aio::MultiplexedConnection;
use proto::bedrock::{MovePlayer, TextData, TextMessage};
use util::{BinaryWrite, MutableBuffer};

#[cfg(test)] mod test;

const TRANS_GEO_KEY: &str = "geotrans";
const XUID_MAP_KEY: &str = "xuidmap";
const TEXT_KEY: &str = "text";

#[derive(Debug)]
struct TextObject {
    xuid: u64,
    src: String,
    body: String
}

impl ToRedisArgs for TextObject {
    fn write_redis_args<W>(&self, out: &mut W) where W: ?Sized + RedisWrite {
        self.xuid.write_redis_args(out);
        self.src.write_redis_args(out);
        self.body.write_redis_args(out);
    }

    fn is_single_arg(&self) -> bool {
        false
    }
}

impl FromRedisValue for TextObject {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        dbg!(v);

        if let Some(seq) = v.as_sequence()  {
            dbg!(seq.len());

            if seq.len() != 6 {
                return Err(RedisError::from(
                    (ErrorKind::ResponseError, "Expected sequence of length 6 as text response")
                ))
            };

            Ok(TextObject {
                xuid: u64::from_redis_value(&seq[3])?,
                src: String::from_redis_value(&seq[4])?,
                body: String::from_redis_value(&seq[5])?
            })
        } else {
            todo!()
        }
    }
}

pub struct Replicator {
    // conn: MultiplexedConnection
}

impl Replicator {
    pub async fn new() -> anyhow::Result<Self> {
        let client = redis::Client::open("redis://replication:6379/")?;
        let conn = client.get_multiplexed_tokio_connection().await?;

        Ok(Self {})

        // Ok(Self {
        //     conn
        // })
    }

    pub async fn save_xuid(&self, xuid: u64, name: String) -> RedisResult<()> {
        Ok(())
        // self.conn.clone().hset(XUID_MAP_KEY, xuid, name).await
    }
    
    pub async fn move_player(&self, xuid: u64, data: &MovePlayer) -> RedisResult<()> {
        Ok(())
        // self.conn
        //     .clone()
        //     .geo_add(TRANS_GEO_KEY, (data.translation.x, data.translation.z, xuid))
        //     .await
    }

    /// Publishes a message to the text chat stream in the replication layer.
    pub async fn text_msg(&self, msg: &TextMessage<'_>) -> RedisResult<()> {
        Ok(())
        // let body = match msg.data {
        //     TextData::Chat { message, source } => {
        //         message
        //     },
        //     _ => todo!()
        // };

        // redis::cmd("XADD")
        //     .arg(TEXT_KEY)
        //     .arg("*")
        //     .arg("xuid").arg(msg.xuid)
        //     .arg("body").arg(body)
        //     .query_async(&mut self.conn.clone())
        //     .await
    }
}

