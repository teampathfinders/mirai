use std::process::ExitCode;
use std::str::FromStr;
use std::sync::atomic::{AtomicU16, Ordering};

use tokio::runtime;

use tracing_subscriber::filter::LevelFilter;

use inferno::instance::ServerInstance;

fn main() -> ExitCode {
    if let Err(e) = init_logging() {
        init_error_log();
        tracing::error!("Failed to initialise tracing: {e}");

        return ExitCode::FAILURE;
    }
    start_server()
}

fn start_server() -> ExitCode {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .thread_name_fn(|| {
            static ATOMIC_THREAD_COUNTER: AtomicU16 = AtomicU16::new(0);
            format!("thread-{}", ATOMIC_THREAD_COUNTER.fetch_add(1, Ordering::Relaxed))
        })
        .build()
        .expect("Failed to build runtime");

    if let Err(error) = runtime.block_on(ServerInstance::run()) {
        tracing::error!("Exited due to fatal error: {error:?}");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

/// Initialises logging with tokio-console.
#[cfg(feature = "tokio-console")]
fn init_logging() -> anyhow::Result<()> {
    use std::time::Duration;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let console_layer = console_subscriber::Builder::default()
        .retention(Duration::from_secs(1))
        .recording_path("console_trace.log")
        .spawn();

    let fmt = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_names(true)
        .with_filter(EnvFilter::from_env("LOG_LEVEL"));

    tracing_subscriber::registry().with(console_layer).with(fmt).init();

    tracing::info!("Tokio console enabled");

    Ok(())
}

/// Initialises logging without tokio-console.
#[cfg(not(feature = "tokio-console"))]
fn init_logging() -> anyhow::Result<()> {
    let max_level = LevelFilter::from_str(
        &std::env::vars()
            .find_map(|(k, v)| if k == "LOG_LEVEL" { Some(v) } else { None })
            .unwrap_or(String::from("info")),
    )?;

    tracing_subscriber::fmt()
        .with_max_level(max_level)
        .with_target(false)
        .with_thread_names(true)
        .init();

    Ok(())
}

fn init_error_log() {
    tracing_subscriber::fmt().with_target(false).with_max_level(LevelFilter::ERROR).init();
}
