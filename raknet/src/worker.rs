use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use tokio::sync::mpsc;


use util::MutableBuffer;

use crate::RaknetUser;

/// Tick interval of the internal session tick.
const INTERNAL_TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);
/// Inactivity timeout.
///
/// Any sessions that do not respond within this specified timeout will be disconnect from the server.
/// Timeouts can happen if a client's game crashed for example.
/// They will stop responding to the server, but will not explicitly send a disconnect request.
/// Hence, they have to be disconnected manually after the timeout passes.
const SESSION_TIMEOUT: Duration = Duration::from_secs(5);

impl RaknetUser {
    /// Starts the ticker task which takes care of packet submission and general user management.
    pub async fn async_worker(
        self: Arc<Self>, mut receiver: mpsc::Receiver<MutableBuffer>
    ) {
        let mut interval = tokio::time::interval(INTERNAL_TICK_INTERVAL);

        let mut should_run = true;
        while should_run {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(err) = self.tick().await {
                        tracing::error!("{err:#}");
                    }
                },
                pk = receiver.recv() => {
                    if let Some(pk) = pk {
                        if let Err(err) = self.handle_raw_packet(pk).await {
                            tracing::error!("{err:#}");
                        }
                    }
                }
            }

            should_run = !self.active.is_cancelled();
        }

        tracing::debug!("Raknet worker closed");
    }

    /// Performs tasks not related to packet processing
    pub async fn tick(&self) -> anyhow::Result<()> {
        let _current_tick = self.tick.fetch_add(1, Ordering::SeqCst);

        // Session has timed out
        if Instant::now().duration_since(*self.last_update.read())
            > SESSION_TIMEOUT
        {
            self.active.cancel();
        }

        self.flush().await?;
        Ok(())
    }
}
