mod error;
mod network;

use std::sync::atomic::{AtomicU16, Ordering};
use std::time::Duration;

use tokio::runtime;

use error::VexResult;
use network::NetworkManager;

const IPV4_PORT: u16 = 19132;
const IPV6_PORT: u16 = 19133;

async fn app_main() -> VexResult<()> {
    loop {
        match NetworkManager::start(IPV4_PORT, Some(IPV6_PORT)).await {
            Ok(_) => {
                tracing::info!("Received OK, not restarting server");
                break;
            }
            Err(e) => {
                tracing::error!("Seems like the server crashed, attempting to restart it...");
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
                "worker-{}",
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
        .with_thread_names(true)
        .pretty();

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
        .pretty()
        .init();

    tracing::info!("Tracing (without console) enabled");
}
