#![allow(dead_code)]

use std::sync::atomic::{AtomicU16, Ordering};

use anyhow::Context;
use tokio::runtime;

use inferno::instance::Instance;
use util::Joinable;

fn main() -> anyhow::Result<()> {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .thread_name_fn(|| {
            static THREAD_COUNTER: AtomicU16 = AtomicU16::new(1);
            format!("worker-{}", THREAD_COUNTER.fetch_add(1, Ordering::Relaxed))
        })
        .build()
        .expect("Failed to build runtime");

    init_logging().context("Unable to initialise logging")?;

    let host = std::env::vars().find_map(|(k, v)| if k == "REDIS_HOST" { Some(v) } else { None });

    let host = if let Some(host) = host {
        host
    } else {
        tracing::debug!("No REDIS_HOST environment variable found, using default host 127.0.0.1");
        String::from("127.0.0.1")
    };

    let port = std::env::vars().find_map(|(k, v)| if k == "REDIS_PORT" { Some(v) } else { None });

    let port: u16 = if let Some(port) = port {
        port.parse().context("Failed to parse REDIS_PORT argument")?
    } else {
        tracing::debug!("No REDIS_PORT environment variable found, using default port 6379");
        6379
    };

    let builder = Instance::builder()
        .database_host(host)
        .database_port(port);

    runtime.block_on(async move {
        let instance = builder.build().await?;
        if let Err(err) = instance.start() {
            tracing::error!("Failed to start server: {err:#}");
            return Err(err);
        }

        instance.join().await
    })
}

/// Initialises logging with tokio-console.
#[cfg(feature = "tokio-console")]
fn init_logging() -> anyhow::Result<()> {
    use std::time::Duration;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::{EnvFilter, Layer};

    let console_layer = console_subscriber::Builder::default()
        .retention(Duration::from_secs(1))
        .recording_path("logs/async.log")
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
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;
    use tracing_subscriber::EnvFilter;

    let requested_level = std::env::vars()
        .find_map(|(k, v)| if k == "LOG_LEVEL" { Some(v) } else { None })
        .unwrap_or(String::from("info"));

    let env_filter = EnvFilter::new(format!("inferno={requested_level}"));

    let fmt = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(true)
        .with_line_number(false)
        .with_file(false)
        .without_time()
        .pretty();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt)
        .try_init()
        .context("Failed to register tracing subscriber")?;

    Ok(())
}
