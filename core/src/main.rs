use std::sync::atomic::{AtomicU16, Ordering};

use tokio::runtime;

use pyro::instance::ServerInstance;

fn main() {
    init_logging();
    init_runtime();
}

fn init_runtime() {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .thread_name_fn(|| {
            static ATOMIC_THREAD_COUNTER: AtomicU16 = AtomicU16::new(0);
            format!("async-thread-{}", ATOMIC_THREAD_COUNTER.fetch_add(1, Ordering::Relaxed))
        })
        .build()
        .expect("Failed to build runtime");

    if let Err(error) = runtime.block_on(ServerInstance::run()) {
        tracing::error!("Fatal error: {error:?}");
    }
}

// /// The Cranelift compiler inside of Wasmer logs verbose things to the console when the level is set
// /// to info. This function disables that.
// fn disable_wasmer_log() {
//     if std::env::var_os("RUST_LOG").is_none() {
//         std::env::set_var("RUST_LOG", "info,wasmer_compiler_cranelift=warn");
//     }
// }

/// Initialises logging with tokio-console.
#[cfg(feature = "tokio-console")]
fn init_logging() {
    use std::time::Duration;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    let console_layer = console_subscriber::Builder::default()
        .retention(Duration::from_secs(1))
        .recording_path("console_trace.log")
        .spawn();

    let fmt = tracing_subscriber::fmt::layer().with_target(false);

    tracing_subscriber::registry().with(console_layer).with(fmt).init();

    tracing::info!("Tokio console enabled");
}

/// Initialises logging without tokio-console.
#[cfg(not(feature = "tokio-console"))]
fn init_logging() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(tracing::Level::DEBUG)
        .pretty()
        .init();
}
