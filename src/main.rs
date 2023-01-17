#![allow(dead_code)]

mod config;
mod error;
mod raknet;
mod services;
mod util;

use std::sync::atomic::{AtomicU16, Ordering};

use tokio::runtime;

use crate::config::ServerConfig;
use crate::services::ServerInstance;
use error::VexResult;

const IPV4_PORT: u16 = 19132;
const IPV6_PORT: u16 = 19133;

const CONFIG: ServerConfig = ServerConfig {
    max_players: 10,
    ipv4_port: 19132,
};

/// The asynchronous entrypoint that is ran by Tokio.
async fn app_main() -> VexResult<()> {
    loop {
        let controller = ServerInstance::new(CONFIG).await?;
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

/// Initialises logging with tokio-console.
#[cfg(feature = "tokio-console")]
fn init_logging() {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use std::time::Duration;

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

/// Initialises logging without tokio-console.
#[cfg(not(feature = "tokio-console"))]
fn init_logging() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .with_thread_names(true)
        .init();

    tracing::info!("Tracing (without console) enabled");
}
