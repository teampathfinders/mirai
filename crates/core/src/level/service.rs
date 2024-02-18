// use std::{any::TypeId, sync::{Arc, OnceLock, Weak}};

// use dashmap::DashMap;
// use proto::types::Dimension;
// use tokio::sync::{mpsc, oneshot};
// use tokio_util::sync::CancellationToken;
// use util::{Joinable, Vector};

// use crate::instance::Instance;

// use super::{gamerule, Rule, RuleValue};

// pub struct RadiusRequest {
//     pub dimension: Dimension,
//     pub center: Vector<i32, 2>,
//     pub radius: u16
// }

// impl Request for RadiusRequest {
//     type Output = ();
// }

// pub struct SingleRequest {
//     pub dimension: Dimension,
//     pub position: Vector<i32, 3>
// }

// impl Request for SingleRequest {
//     type Output = ();
// }

// mod private {
//     pub trait Sealed {}
//     impl Sealed for super::SingleRequest {}
//     impl Sealed for super::RadiusRequest {}
//     impl Sealed for super::RegionRequest {}
// }

// pub struct ServiceRequest {
    
// }

// pub trait Request {
//     type Output;

//     fn execute(&self, service: &Arc<Service>) -> mpsc::Receiver<Self::Output> {
//         let (sender, receiver) = mpsc::channel();
//         receiver
//     }
// }

use std::{any::TypeId, sync::{Arc, OnceLock, Weak}};

use dashmap::DashMap;
use level::{provider::Provider, DataKey, KeyType, SubChunk};
use proto::types::Dimension;
use rayon::iter::{IndexedParallelIterator, ParallelIterator};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use util::{Joinable, Vector};

use crate::{command::ServiceRequest, instance::Instance};

use super::{IndexedSubChunk, RegionIndex, RegionStream, Rule, RuleValue};

/// Types that can be used in region requests.
pub trait IntoRegion: Send + Sync + 'static {
    /// Iterator that this region can be turned into.
    type IntoIter: IndexedParallelIterator<Item = Vector<i32, 3>>;

    /// Creates an iterator over this region.
    fn iter(&self, provider: Arc<Provider>) -> Self::IntoIter;
    /// Converts a coordinate to an index into this region.
    fn as_index(&self, coord: &Vector<i32, 3>) -> usize;
    /// Converts an index to a coordinate into this region.
    fn as_coord(&self, index: usize) -> Vector<i32, 3>;
}

pub(crate) struct ServiceOptions {
    pub instance_token: CancellationToken,
    pub level_path: String
}

pub struct Service {
    instance_token: CancellationToken,
    shutdown_token: CancellationToken,
    instance: OnceLock<Weak<Instance>>,
    /// Provides level data from disk.
    pub(super) provider: Arc<level::provider::Provider>,
    /// Current gamerule values.
    /// The gamerules are stored by TypeId to allow for user-defined gamerules.
    gamerules: DashMap<TypeId, RuleValue>,

    sender: mpsc::Sender<ServiceRequest>
}

impl Service {
    pub(crate) fn new(
        options: ServiceOptions
    ) -> anyhow::Result<Arc<Service>> {
        let provider = Arc::new(unsafe {
            level::provider::Provider::open(&options.level_path)
        }?);

        let (sender, receiver) = mpsc::channel(10);
        let service = Arc::new(Service {
            instance_token: options.instance_token,
            shutdown_token: CancellationToken::new(),
            instance: OnceLock::new(),

            provider,
            gamerules: DashMap::new(),
            sender
        });

        let this = Arc::clone(&service);
        tokio::spawn(this.service(receiver));

        Ok(service)
    }

    /// Sets the parent instance of this service.
    pub(crate) fn set_instance(&self, instance: &Arc<Instance>) -> anyhow::Result<()> {
        self.instance
            .set(Arc::downgrade(instance))
            .map_err(|_| anyhow::anyhow!("Level service instance was already set"))
    }   

    /// Requests chunks using the specified region iterator.
    pub fn request_region<R: IntoRegion>(self: &Arc<Service>, region: R) -> RegionStream {
        let iter = region.iter(Arc::clone(&self.provider));
        let len = iter.len();
        let (sender, receiver) = mpsc::channel(len);

        let provider = Arc::clone(&self.provider);
        rayon::spawn(move || {
            // If this returns an error, the receiver has closed so we can stop processing.
            let _ = iter   
                .try_for_each(|item| {
                    let subchunk = provider.get_subchunk(
                        Vector::from([item.x, item.z]), item.y as i8, Dimension::Overworld
                    );

                    let subchunk = match subchunk {
                        Ok(Some(chunk)) => chunk,
                        Ok(None) => SubChunk::empty(item.y as i8),
                        Err(e) => {
                            tracing::error!("Failed to load subchunk at {item:?}: {e:#}");
                            SubChunk::empty(item.y as i8)
                        }
                    };

                    let indexed = IndexedSubChunk {
                        index: RegionIndex::from(item),
                        data: subchunk
                    };

                    sender.blocking_send(indexed)
                });
        });

        RegionStream {
            inner: receiver,
            len
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

    async fn service(self: Arc<Service>, mut receiver: mpsc::Receiver<ServiceRequest>) {
        loop {
            tokio::select! {
                request = receiver.recv() => {
                    let Some(request) = request else {
                        // This service is no longer referenced by anyone, shut down.
                        break
                    };
                }
                _ = self.instance_token.cancelled() => break
            }
        }

        self.shutdown_token.cancel();
    }
}

impl Joinable for Service {
    async fn join(&self) -> anyhow::Result<()> {
        self.shutdown_token.cancelled().await;
        Ok(())
    }
}