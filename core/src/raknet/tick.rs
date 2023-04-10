use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};

use tokio::sync::mpsc;

use util::bytes::MutableBuffer;

use crate::network::{
    {PlayerListRemove, TextMessage},
    Session, TextData,
};

use super::{RakNetSession, RakNetMessage};

/// Tick interval of the internal session tick.
const INTERNAL_TICK_INTERVAL: Duration = Duration::from_millis(1000 / 20);
/// Inactivity timeout.
///
/// Any sessions that do not respond within this specified timeout will be disconnect from the server.
/// Timeouts can happen if a client's game crashed for example.
/// They will stop responding to the server, but will not explicitly send a disconnect request.
/// Hence, they have to be disconnected manually after the timeout passes.
const SESSION_TIMEOUT: Duration = Duration::from_secs(5);

impl RakNetSession {
    pub fn start_tick_job(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(INTERNAL_TICK_INTERVAL);

            while self.is_active() {
                match self.tick().await {
                    Ok(_) => (),
                    Err(e) => tracing::error!("{e}"),
                }
                interval.tick().await;
            }

            // Flush last acknowledgements before closing
            match self.flush_acknowledgements().await {
                Ok(_) => (),
                Err(_e) => {
                    tracing::error!(
                        "Failed to flush last acknowledgements before session close"
                    );
                }
            }

            // Flush last packets before closing
            match self.flush().await {
                Ok(_) => (),
                Err(_e) => {
                    tracing::error!(
                        "Failed to flush last packets before session close"
                    );
                }
            }
        });
    }

    pub fn start_packet_job(
        self: Arc<Self>,
        mut receiver: mpsc::Receiver<MutableBuffer>,
    ) {
        tokio::spawn(async move {
            let mut broadcast = self.broadcast.subscribe();

            while self.is_active() {
                tokio::select! {
                    packet = receiver.recv() => {
                        if let Some(packet) = packet {
                            match self.process_raw_packet(packet).await {
                                Ok(_) => (),
                                Err(e) => tracing::error!("{e}"),
                            }
                        }
                    },
                    packet = broadcast.recv() => {
                        if let Ok(packet) = packet {
                            match self.process_broadcast(packet) {
                                Ok(_) => (),
                                Err(e) => tracing::error!("{e}"),
                            }
                        }
                    }
                }
                ;
            }
        });
    }

    /// Signals to the session that it needs to close.
    pub async fn on_disconnect(&self) {
        if !self.is_active() {
            return;
        }

        self.sender.send(RakNetMessage::Disconnect).await.unwrap();
        self.token.cancel();
    }

    /// Performs tasks not related to packet processing
    pub async fn tick(&self) -> anyhow::Result<()> {
        let current_tick = self.current_tick.fetch_add(1, Ordering::SeqCst);

        // Session has timed out
        if Instant::now().duration_since(*self.last_update.read())
            > SESSION_TIMEOUT
        {
            self.on_disconnect().await;
        }

        self.flush().await?;
        Ok(())
    }
}
