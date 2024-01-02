#[cfg(test)] mod test;

pub struct SessionTransform {
    translation: util::Vector<f32, 3>,
    rotation: util::Vector<f32, 3>
}

pub struct Replicator {
    con: redis::aio::MultiplexedConnection
}

impl Replicator {
    pub async fn new() -> anyhow::Result<Self> {
        let client = redis::Client::open("redis://127.0.0.1:6379/")?;
        let mut con = client.get_multiplexed_tokio_connection().await?;

        Ok(Self {
            con
        })
    }
}

