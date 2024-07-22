use std::{any::TypeId, sync::{Arc, OnceLock, Weak}};

use dashmap::DashMap;
use level::{provider::Provider, SubChunk};
use proto::types::Dimension;
use rayon::iter::ParallelIterator;
use tokio::sync::mpsc::{self, error::SendError};
use tokio_util::sync::CancellationToken;
use util::{Joinable, Vector};

use crate::instance::Instance;

use super::{Collector, IndexedSubChunk, Region, RegionIndex, RegionSink, RegionStream, Rule, RuleValue};

pub struct ServiceOptions {
    pub instance_token: CancellationToken,
    pub level_path: String
}

/// Threshold for the service to switch from singular to batching mode.
/// Any requests with more chunks than specified in this threshold will be processed
/// with a parallel iterator and threadpool.
const REGION_PARALLEL_THRESHOLD: usize = 100;

/// Manages the world of the server.
pub struct Service {
    /// Cancelled when the whole server is shutting down. This will then signal to this
    /// service to shut down as well.
    instance_token: CancellationToken,
    /// Cancelled once this service has fully shut down.
    shutdown_token: CancellationToken,
    /// Reference to the parent instance.
    instance: OnceLock<Weak<Instance>>,
    /// Provides level data from disk.
    provider: Arc<level::provider::Provider>,
    /// Collects subchunk changes using sinks and writes them to disk periodically.
    collector: Collector,
    /// Current gamerule values.
    /// The gamerules are stored by TypeId to allow for user-defined gamerules.
    gamerules: DashMap<TypeId, RuleValue>,
}

impl Service {
    pub(crate) fn new(
        options: ServiceOptions
    ) -> anyhow::Result<Arc<Service>> {
        let provider = Arc::new(unsafe {
            level::provider::Provider::open(&options.level_path)
        }?);

        let service = Arc::new(Service {
            collector: Collector::new(Arc::clone(&provider), options.instance_token.clone(), 100),
            instance_token: options.instance_token,
            shutdown_token: CancellationToken::new(),
            instance: OnceLock::new(),
            provider,
            gamerules: DashMap::new()
        });
        Ok(service)
    }

    /// Sets the parent instance of this service.
    pub(crate) fn set_instance(&self, instance: &Arc<Instance>) -> anyhow::Result<()> {
        self.instance
            .set(Arc::downgrade(instance))
            .map_err(|_| anyhow::anyhow!("Level service instance was already set"))
    }   

    /// Requests chunks using the specified region iterator.
    pub fn region<R: Region>(self: &Arc<Service>, region: R) -> RegionStream 
    where
        R::IntoIter: Send
    {
        if region.len() >= REGION_PARALLEL_THRESHOLD {
            self.request_parallel_region(region)
        } else {
            self.request_sequential_region(region)
        }
    }

    /// Creates a new [`RegionSink`]. A region sink allows you to save modified subchunks to disk.
    pub fn region_sink(&self) -> RegionSink {
        self.collector.create_sink()
    }

    /// Loads a region using a parallel region.
    /// 
    /// This function is used for smaller regions that do not benefit from
    /// parallel processing.
    fn request_sequential_region<R: Region>(&self, region: R) -> RegionStream
    where
        R::IntoIter: Send
    {
        let len = region.len();
        let dim = region.dimension();
        let mut iter = region.into_iter();

        let (sender, receiver) = mpsc::channel(len);

        let provider = Arc::clone(&self.provider);
        tokio::task::spawn_blocking(move || {
            // If this returns an error, the receiver has closed so we can stop processing.
            let _: Result<(), SendError<IndexedSubChunk>> = iter
                .try_for_each(|item| {
                    let indexed = Self::for_each_subchunk(item, dim, &provider);
                    sender.blocking_send(indexed)
                });
        });

        RegionStream {
            inner: receiver,
            len
        }
    }

    /// Loads a region using a parallel iterator.
    /// 
    /// This function processes multiple subchunks in parallel by spreading
    /// processing of the region over multiple threads.
    /// 
    /// On my machine a region of 1000 chunks benefits from a 3x speed up.
    /// The more chunks that are requested at once, the better the speed up.
    /// 
    /// The parallel iterator is not used for small regions because the overhead is larger
    /// than the performance boost for small regions.
    fn request_parallel_region<R: Region>(&self, region: R) -> RegionStream {
        let len = region.len();
        let dim = region.dimension();
        let iter = region.into_par_iter();
        let (sender, receiver) = mpsc::channel(len);

        let provider = Arc::clone(&self.provider);
        rayon::spawn(move || {
            // If this returns an error, the receiver has closed so we can stop processing.
            let _: Result<(), SendError<IndexedSubChunk>> = iter   
                .try_for_each(|item| {
                    let indexed = Self::for_each_subchunk(item, dim, &provider);
                    sender.blocking_send(indexed)
                });
        });

        RegionStream {
            inner: receiver,
            len
        }
    }

    /// Operation performed on each subchunk. This is put into a separate function because both
    /// the sequential and parallel iterator perform the exact same operations.
    #[inline]
    fn for_each_subchunk(item: Vector<i32, 3>, dimension: Dimension, provider: &Provider) -> IndexedSubChunk {
        let subchunk = provider.subchunk(
            Vector::from([item.x, item.z]), item.y as i8, dimension
        );

        let subchunk = match subchunk {
            Ok(Some(chunk)) => chunk,
            Ok(None) => SubChunk::empty(item.y as i8),
            Err(e) => {
                tracing::error!("Failed to load subchunk at {item:?}: {e:#}. Replacing it with an empty one...");
                SubChunk::empty(item.y as i8)
            }
        };

        IndexedSubChunk {
            index: RegionIndex::from(item),
            data: subchunk
        }
    }

    /// Sets the value of the given gamerule, returning the old value.
    /// 
    /// Instead of referring to the gamerules by name, I decided to use generics instead.
    /// So for example if you wanted to change the value of the `tntexplodes` gamerule in a command handler, you would do
    /// it like this:
    /// ```ignore
    /// let old_value = ctx.instance.level().set_gamerule::<TntExplodes>(true);
    /// ```
    /// 
    /// See [`Rule`] for defining your own custom gamerules.
    pub fn set_gamerule<R: Rule>(&self, value: R::Value) -> R::Value
        where RuleValue: From<R::Value> // Ensure that the gamerule has a valid value type.
    {
        let value = RuleValue::from(value);
        let old = self.gamerules.insert(TypeId::of::<R>(), value);
        
        let Some(old) = old else {
            return R::Value::default()
        };

        old.into()
    }

    /// Returns the value of the given gamerule.
    /// 
    /// Instead of referring to the gamerules by name, I decided to use generics instead.
    /// So for example if you wanted to read the value of the `tntexplodes` gamerule in a command handler, you would do
    /// it like this:
    /// ```ignore
    /// let value = ctx.instance.level().gamerule::<TntExplodes>();
    /// ```
    /// 
    /// See [`Rule`] for defining your own custom gamerules.
    pub fn gamerule<R: Rule>(&self) -> R::Value
        where RuleValue: From<R::Value> // Ensure that the gamerule has a valid value type.
    {
        let Some(kv) = self.gamerules.get(&TypeId::of::<R>()) else {
            return R::Value::default()
        };

        (*kv.value()).into()
    }
}

impl Joinable for Service {
    async fn join(&self) -> anyhow::Result<()> {
        self.collector.join().await?;

        Ok(())
    }
}