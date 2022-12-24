use std::sync::atomic::{AtomicU16, Ordering};

use tokio::runtime;

async fn app_main() {
    println!("Hello World!");
}

fn main() {
    let runtime = runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name_fn(|| {
            static ATOMIC_THREAD_COUNTER: AtomicU16 = AtomicU16::new(0);
            format!("worker-{}", ATOMIC_THREAD_COUNTER.fetch_add(1, Ordering::Relaxed))
        })
        .build().expect("Failed to build runtime");

    runtime.block_on(app_main())
}