use redis::{ErrorKind, FromRedisValue, RedisError, RedisResult, RedisWrite, ToRedisArgs, Value};
use proto::bedrock::{TextData, TextMessage};

#[cfg(test)] mod test;

const TEXT_STREAM: &str = "text:chat";

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
}

impl FromRedisValue for TextObject {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match *v {
            Value::Nil => Err(RedisError::from(
                (ErrorKind::ResponseError, "Text stream response object cannot be empty"))
            ),
            _ => {
                dbg!(v);
                todo!()
            }
        }
    }
}

pub struct Replicator {
    conn: redis::aio::MultiplexedConnection
}

impl Replicator {
    pub async fn new() -> anyhow::Result<Self> {
        let client = redis::Client::open("redis://127.0.0.1:6379/")?;
        let mut conn = client.get_multiplexed_tokio_connection().await?;

        Ok(Self { conn })
    }

    pub fn sub_text_message(&self) {
        let mut conn = self.conn.clone();
        tokio::spawn(async move {
            loop {
                println!("hello");

                let msg: RedisResult<TextObject> = redis::cmd("XREAD")
                    .arg("BLOCK")
                    .arg(0)
                    .arg("STREAMS")
                    .arg(TEXT_STREAM)
                    .arg("$")
                    .query_async(&mut conn)
                    .await;

                match msg {
                    Ok(msg) => { dbg!(msg); },
                    Err(e) => { tracing::error!("{:?}", e.detail()); }
                }
            }
        });
    }

    /// Publishes a message to the text chat stream in the replication layer.
    pub fn pub_text_message(&self, msg: &TextMessage) {
        let mut conn = self.conn.clone();
        let (body, src) = match msg.data {
            TextData::Chat { message, source } => {
                (message, source)
            },
            _ => todo!()
        };

        let xuid = msg.xuid;
        let (body, src) = (body.to_owned(), src.to_owned());

        let obj = TextObject {
            xuid, body, src
        };

        dbg!(&obj);

        tokio::spawn(async move {
            // let _: () = redis::cmd("XADD")
            //     .arg(TEXT_STREAM).arg("*")
            //     .arg("xuid").arg(xuid)
            //     .arg("src").arg(src)
            //     .arg("body").arg(body)
            //     .query_async(&mut conn)
            //     .await
            //     .unwrap();

            let _: () = redis::cmd("XADD")
                .arg(TEXT_STREAM)
                .arg("*")
                .arg("data")
                .arg(obj)
                .query_async(&mut conn)
                .await
                .unwrap();

            println!("Saved chat message");
        });
    }
}

