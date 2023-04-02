use std::sync::Arc;
use std::time::Duration;
use parking_lot::RwLock;
use tokio_util::sync::CancellationToken;

pub struct ChunkManager {
    level_dat: RwLock<level::LevelDat>,
    /// Chunk database
    database: level::Database,
    token: CancellationToken,
}

impl ChunkManager {
    pub fn new<P: AsRef<str>>(
        path: P,
        autosave_interval: Duration,
        token: CancellationToken,
    ) -> anyhow::Result<(Arc<Self>, Receiver<()>)> {
        tracing::info!("Loading level {}...", path.as_ref());

        let manager = Arc::new(Self {
            level_dat,
            database: level::Database::open(path)?,
            token
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
    pub fn flush(&self) -> anyhow::Result<()> {
        Ok(())
        // todo!();
    }
}