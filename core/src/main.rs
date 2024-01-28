use std::sync::atomic::{AtomicU16, Ordering};

use anyhow::Context;
use tokio::runtime;

use tracing_subscriber::filter::LevelFilter;

use inferno::instance::{Instance, InstanceBuilder, NetConfig};

#[cfg(unix)]
fn main() -> anyhow::Result<()> {
    use pyroscope::PyroscopeAgent;
    use pyroscope_pprofrs::{pprof_backend, PprofConfig};

    let pprof_config = PprofConfig::new().sample_rate(100);
    let backend_impl = pprof_backend(pprof_config);

    let agent = PyroscopeAgent::builder("http://localhost:4040", "inferno")
        .backend(backend_impl)
        .tags(vec![("TagA", "ValueA")])
        .build()?;

    let agent = agent.start().unwrap();
    if let Err(err) = init_logging().context("Unable to initialise logging") {
        agent.shutdown();
        return Err(err);
    }
    tracing::debug!("Telemetry enabled");

    let code = start_server();
    let agent = agent.stop()?;
    agent.shutdown();
    code
}

#[cfg(not(unix))]
fn main() -> anyhow::Result<()> {
    init_logging().context("Unable to initialise logging")?;
    tracing::debug!("Telemetry disabled");

    start_server()
}

fn start_server() -> anyhow::Result<()> {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .thread_name_fn(|| {
            static THREAD_COUNTER: AtomicU16 = AtomicU16::new(1);
            format!("[{}]", THREAD_COUNTER.fetch_add(1, Ordering::Relaxed))
        })
        .build()
        .expect("Failed to build runtime");

    let builder = InstanceBuilder::new().net_config(NetConfig {
        max_connections: 10,
        ..Default::default()
    });

    runtime.block_on(async move {
        let instance = builder.build().await?;
        instance.run().await
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
    use std::str::FromStr;

    let max_level = LevelFilter::from_str(
        &std::env::vars()
            .find_map(|(k, v)| if k == "LOG_LEVEL" { Some(v) } else { None })
            .unwrap_or(String::from("info")),
    )?;

    tracing_subscriber::fmt()
        .with_max_level(max_level)
        .with_target(false)
        .with_thread_names(true)
        .with_file(true)
        .pretty()
        .init();

    Ok(())
}
