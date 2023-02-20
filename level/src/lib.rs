#[cfg(test)]
mod test;

mod database;
mod ffi;
mod sub_chunk;
mod world;

use std::{sync::Arc, time::Duration};

use common::VResult;
use database::ChunkDatabase;
pub use sub_chunk::*;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio_util::sync::CancellationToken;
pub use world::*;

/// Interface that is used to read and write world data.
#[derive(Debug)]
pub struct ChunkManager {
    /// Chunk database
    database: ChunkDatabase,
    token: CancellationToken,
}

impl ChunkManager {
    pub fn new<P: AsRef<str>>(
        path: P,
        autosave_interval: Duration,
        token: CancellationToken,
    ) -> VResult<(Arc<Self>, Receiver<()>)> {
        tracing::info!("Loading level {}...", path.as_ref());

        let manager = Arc::new(Self {
            database: ChunkDatabase::new(path)?,
            token,
        });

        let clone = manager.clone();
        let (sender, receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            clone.autosave_job(sender, autosave_interval).await
        });

        Ok((manager, receiver))
    }

    /// Writes the current level state to the disk.
    /// Internally, this uses LevelDB's WriteBatch method to perform bulk updates.
    /// These LevelDB are done synchronously to prevent data loss and the overhead is minimal due to batching.
    pub fn flush(&self) -> VResult<()> {
        Ok(())
    }

    /// Simple job that runs [`flush`](Self::flush) on a specified interval.
    async fn autosave_job(&self, sender: Sender<()>, interval: Duration) {
        let mut interval = tokio::time::interval(interval);

        // Run until there are no more references to the chunk manager.
        // (other than this job).
        //
        // This prevents a memory leak in case someone drops the chunk manager.
        loop {
            match self.flush() {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to save level: {e}");
                }
            }

            tokio::select! {
                _ = interval.tick() => (),
                _ = self.token.cancelled() => break
            };
        }

        // Save before closing.
        match self.flush() {
            Ok(_) => (),
            Err(e) => {
                tracing::error!("Failed to save level: {e}");
            }
        }
        drop(self);

        // Send the signal that the level has been closed.
        let _ = sender.send(());
        tracing::info!("Closed level");
    }
}
