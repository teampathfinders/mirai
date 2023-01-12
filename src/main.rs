mod config;
mod controller;
mod error;
mod raknet;
mod util;
mod worker;

use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use tokio::runtime;

use crate::config::ServerConfig;
use crate::controller::ServerController;
use error::VexResult;
use raknet::NetController;

const IPV4_PORT: u16 = 19132;
const IPV6_PORT: u16 = 19133;

const CONFIG: ServerConfig = ServerConfig {
    max_players: 10,
    ipv4_port: 19132,
};

async fn app_main() -> VexResult<()> {
    loop {
        let controller = ServerController::new(CONFIG).await?;
        match controller.run().await {
            Ok(_) => {
                tracing::info!("Received OK for shutdown, not restarting controller");
                break;
            }
            Err(e) => {
                tracing::error!("Seems like the controller panicked, attempting to restart it...");
                tracing::error!("Crash cause: {e:?}");
            }
        }
    }

    Ok(())
}

fn main() -> VexResult<()> {
    init_logging();

    let runtime = runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .thread_name_fn(|| {
            static ATOMIC_THREAD_COUNTER: AtomicU16 = AtomicU16::new(0);
            format!(
                "async-thread-{}",
                ATOMIC_THREAD_COUNTER.fetch_add(1, Ordering::Relaxed)
            )
        })
        .build()
        .expect("Failed to build runtime");

    runtime.block_on(app_main())
}

#[cfg(feature = "tokio-console")]
fn init_logging() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let console_layer = console_subscriber::Builder::default()
        .retention(Duration::from_secs(1))
        .recording_path("console_trace.log")
        .spawn();

    let fmt = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_names(true);

    tracing_subscriber::registry()
        .with(console_layer)
        .with(fmt)
        .init();

    tracing::info!("Tracing (with console) enabled");
}

#[cfg(not(feature = "tokio-console"))]
fn init_logging() {
    use tracing::Level;
    use tracing_subscriber::util::SubscriberInitExt;

    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_names(true)
        .init();

    tracing::info!("Tracing (without console) enabled");
}
