use std::sync::atomic::{AtomicU16, Ordering};

use anyhow::Context;
use tokio::runtime::{self, Handle};

use lazy_static::lazy_static;
use prometheus_client::registry::Registry;

use inferno::instance::{DbConfig, InstanceBuilder, NetConfig};

mod metrics;

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

    let handle = runtime.handle();

    init_logging(handle).context("Unable to initialise logging")?;

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

    let builder = InstanceBuilder::new()
        .net_config(NetConfig {
            max_connections: 10,
            ..Default::default()
        })
        .db_config(DbConfig { host: &host, port });

    let out = runtime.block_on(async move {
        tokio::spawn(async move { metrics::metrics_agent().await });

        let instance = builder.build().await?;
        instance.run().await
    });

    // opentelemetry::global::shutdown_tracer_provider();

    out
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
fn init_logging(_handle: &Handle) -> anyhow::Result<()> {
    use std::str::FromStr;
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let level_filter = LevelFilter::from_str(
        &std::env::vars()
            .find_map(|(k, v)| if k == "LOG_LEVEL" { Some(v) } else { None })
            .unwrap_or(String::from("info")),
    )?;

    let fmt = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_ansi(true)
        .with_line_number(false)
        .with_file(false)
        .without_time()
        .pretty();

    tracing_subscriber::registry()
        .with(level_filter)
        .with(fmt)
        .try_init()
        .context("Failed to register tracing subscriber")?;

    Ok(())
}
