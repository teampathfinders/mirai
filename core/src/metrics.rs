use prometheus_client::{encoding::text::encode, registry::Registry};
use tokio::{
    io::{self, Interest},
    net::TcpListener,
};

use std::sync::Arc;

fn register_metrics() -> Registry {
    use inferno::net::{CONNECTED_CLIENTS_METRIC, RESPONSE_TIMES_METRIC};
    use raknet::TOTAL_PACKETS_METRIC;

    let mut registry = <Registry>::default();

    registry.register("raknet_total_packets", "Number of UDP packets received", TOTAL_PACKETS_METRIC.clone());
    registry.register("connected_clients", "Amount of connected clients", CONNECTED_CLIENTS_METRIC.clone());
    registry.register(
        "response_times",
        "Response times of the upper service layer",
        RESPONSE_TIMES_METRIC.clone(),
    );

    registry
}

#[tracing::instrument]
pub async fn metrics_agent() -> anyhow::Result<()> {
    let registry = Arc::new(register_metrics());
    let listener = TcpListener::bind("0.0.0.0:9090").await?;

    let (stream, _) = listener.accept().await?;
    let mut recv_buf = vec![0; 1024];

    loop {
        let ready = stream.ready(Interest::READABLE | Interest::WRITABLE).await?;
        if ready.is_readable() {
            recv_buf.clear();

            let data;
            match stream.try_read(&mut recv_buf) {
                Ok(n) => {
                    data = &recv_buf[..n];
                    todo!()
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Wakeup was a false positive.
                    continue;
                }
                Err(e) => {
                    tracing::error!("Unable to read metrics socket: {e}");
                }
            }
        }
    }
}

fn collect_metrics(registry: Arc<Registry>) -> anyhow::Result<()> {
    let mut buffer = String::new();
    encode(&mut buffer, &registry)?;

    tracing::debug!("{buffer}");

    Ok(())
}
