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
pub use world::*;

/// Interface that is used to read and write world data.
#[derive(Debug)]
pub struct ChunkManager {
    /// Chunk database
    database: ChunkDatabase,
}

impl ChunkManager {
    pub fn new<P: AsRef<str>>(path: P, autosave_interval: Duration) -> VResult<Arc<Self>> {
        let manager = Arc::new(Self {
            database: ChunkDatabase::new(path)?
        });

        let clone = manager.clone();
        tokio::spawn(async move {
            clone.autosave_job(autosave_interval).await
        });

        Ok(manager)
    }

    /// Writes the current level state to the disk.
    /// Internally, this uses LevelDB's WriteBatch method to perform bulk updates.
    /// These LevelDB are done synchronously to prevent data loss and the overhead is minimal due to batching.
    pub fn flush(&self) -> VResult<()> {
        todo!("Implement chunk flush.");

        Ok(())
    }   

    /// Simple job that runs [`flush`](Self::flush) on a specified interval.
    async fn autosave_job(self: Arc<Self>, interval: Duration) {
        let mut interval = tokio::time::interval(interval);

        // Run until there are no more references to the chunk manager.
        // (other than this job).
        //
        // This prevents a memory leak in case someone drops the database.
        while Arc::strong_count(&self) > 1 {
            match self.flush() {
                Ok(_) => (),
                Err(e) => {
                    tracing::error!("Failed to save level: {e}");
                }
            }
            interval.tick().await;
        }
    }
}
