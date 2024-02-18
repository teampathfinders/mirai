use std::{
    sync::{Arc, atomic::{AtomicU64, Ordering}},
    time::{Duration, Instant},
};

use prometheus_client::metrics::counter::Counter;
use tokio::sync::{mpsc, TryAcquireError};
use util::RVec;

use crate::{RakNetCommand, RakNetClient};

use lazy_static::lazy_static;

lazy_static! {
    #[doc(hidden)]
    pub static ref TOTAL_PACKETS_METRIC: Counter::<u64, AtomicU64> = Counter::default();
}

/// Limit to the amount of packets a client is allowed to send per second.
pub const BUDGET_SIZE: usize = 50;

/// Tick interval of the internal session tick.
const INTERNAL_TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);
/// Inactivity timeout.
///
/// Any sessions that do not respond within this specified timeout will be disconnect from the server.
/// Timeouts can happen if a client's game crashed for example.
/// They will stop responding to the server, but will not explicitly send a disconnect request.
/// Hence, they have to be disconnected manually after the timeout passes.
const SESSION_TIMEOUT: Duration = Duration::from_secs(5);

impl RakNetClient {
    /// Starts the ticker task which takes care of packet submission and general user management.
    #[tracing::instrument(
        skip_all,
        name = "RaknetUser::receiver",
        fields(
            address = %self.address
        )
    )]
    pub async fn receiver(
        self: Arc<Self>, mut receiver: mpsc::Receiver<RVec>
    ) {
        let mut interval = tokio::time::interval(INTERNAL_TICK_INTERVAL);

        let mut should_run = true;
        let mut has_exhausted = false;

        while should_run {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(err) = self.tick().await {
                        tracing::error!("{err:#}");
                    }
                },
                packet = receiver.recv() => {
                    let Some(packet) = packet else {
                        // Receiver channel closed, shut down this session.
                        break
                    };

                    match self.budget.try_acquire() {
                        Ok(permit) => permit.forget(),
                        Err(TryAcquireError::Closed) => unreachable!(),
                        Err(TryAcquireError::NoPermits) if !has_exhausted => {
                            // Prevent printing error twice.
                            has_exhausted = true;

                            tracing::warn!("Client exhausted its budget. Too many packets have been sent within the last second");

                            // Notify parent of exhausted budget. The parent should then disconnect the client.
                            if self.output.send(RakNetCommand::BudgetExhausted).await.is_err() {
                                // Parent has somehow been lost. This service is useless without a parent, so exit.
                                self.disconnect();
                            }
                        }
                        _ => ()
                    }

                    if let Err(err) = self.handle_raw_packet(packet).await {
                        tracing::error!("{err:?}");
                    }
                    TOTAL_PACKETS_METRIC.inc();
                }
            }

            should_run = !self.active.is_cancelled();
        }

        if let Err(err) = self.flush_all().await {
            tracing::error!("Failed to flush client's final packets: {err:#}");
        }

        self.shutdown_token.cancel();
    }

    /// Performs tasks not related to packet processing
    pub async fn tick(&self) -> anyhow::Result<()> {
        let current_tick = self.tick.fetch_add(1, Ordering::SeqCst);

        // Reset budget every second.
        if current_tick % 20 == 0 {
            // self.budget.add_permits(BUDGET_SIZE - self.budget.available_permits());
            self.refill_budget();
        }

        // Session has timed out
        if Instant::now().duration_since(*self.last_update.read())
            > SESSION_TIMEOUT
        {
            tracing::warn!("Client unresponsive, disconnecting them...");
            self.active.cancel();
        }

        self.flush().await?;
        Ok(())
    }
}
