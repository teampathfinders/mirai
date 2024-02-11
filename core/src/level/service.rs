use std::{any::TypeId, sync::{Arc, OnceLock, Weak}};

use dashmap::{DashMap, DashSet};
use parking_lot::RwLock;
use proto::{bedrock::GameRule, types::Dimension};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use util::Vector;

use crate::instance::Instance;

const LEVEL_REQUEST_BUFFER_SIZE: usize = 10;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RequestableVariant {
    SingleGet,
    MultiGet,
}

pub trait ExpensiveRequestable: Sized + 'static {
    type Output;

    const VARIANT: RequestableVariant;

    /// Casts a generic to the original type.
    /// 
    /// `T` and `Self` must be the exact same type.
    fn cast<T: 'static>(self) -> T {
        assert_eq!(TypeId::of::<T>(), TypeId::of::<Self>(), "Cannot cast requestable to different type");
        
        // SAFETY: This is safe because both types are guaranteed to be the same.
        let cast = unsafe {
            std::mem::transmute_copy::<Self, T>(&self)
        };
        std::mem::forget(self);

        cast
    }
}

/// Loads multiple subchunks around a given center.
#[derive(Debug)]
pub struct SubchunkGetMulti {
    /// Dimension to load the chunks from.
    pub dimension: Dimension,
    /// Center to load the chunks around.
    /// 
    /// This is in subchunk coordinates.
    pub center: Vector<i32, 3>,
    /// Offsets to load.
    /// 
    /// For example, given a center (0, 0, 0) an offset of (0, 1, 0) would
    /// translate to the subchunk located at (0, 1, 0),
    pub offsets: Vec<Vector<i8, 3>>
}

impl ExpensiveRequestable for SubchunkGetMulti {
    type Output = String;
    
    const VARIANT: RequestableVariant = RequestableVariant::MultiGet;
}

/// Loads a single subchunk at the given position and dimension.
/// 
/// If possible, [`SubchunkGetMultiCommand`] should be preferred over this one
/// to load subchunks in batches.
#[derive(Debug)]
pub struct SubchunkGetSingle {
    /// Dimension to load the chunks from.
    pub dimension: Dimension,
    /// Position the subchunk is located at.
    pub position: Vector<i32, 3>
}

impl ExpensiveRequestable for SubchunkGetSingle {
    type Output = String;

    const VARIANT: RequestableVariant = RequestableVariant::SingleGet;
}

pub struct ServiceResult {}

pub enum ServiceCommand {
    Multi(SubchunkGetMulti),
    Single(SubchunkGetSingle)
}

struct ServiceRequest {
    command: ServiceCommand,
    callback: oneshot::Sender<ServiceResult>
}

/// Some simpler functionality (such as gamerules) is done using shared state while the more expensive computation
/// such as subchunks is done with message passing. The message passing system ensures that a session can continue handling
/// packets while a subchunk is loading.
/// 
/// All subchunk data is stored on the heap, it is therefore cheap to move subchunks through channels.
pub struct Service {
    token: CancellationToken,

    // gamerules: RwLock<[GameRule; GameRule::variant_count()]>,

    requests: mpsc::Receiver<ServiceRequest>,
    request_producer: mpsc::Sender<ServiceRequest>,
    instance: OnceLock<Weak<Instance>>
}

impl Service {
    pub fn new(token: CancellationToken) -> Arc<Service> {
        let (sender, receiver) = mpsc::channel(LEVEL_REQUEST_BUFFER_SIZE);
        Arc::new(Service {
            token,

            // gamerules: RwLock::new(Default::default()),

            requests: receiver,
            request_producer: sender,
            instance: OnceLock::new()
        })
    }

    /// Sets the instance pointer for this service.
    /// 
    /// This is used to access data from other services.
    pub(crate) fn set_instance(&self, instance: &Arc<Instance>) -> anyhow::Result<()> {
        self.instance.set(Arc::downgrade(instance)).map_err(|_| anyhow::anyhow!("Level service instance was already set"))
    }
    
    pub fn get<R: ExpensiveRequestable>(&self, request: R) -> anyhow::Result<oneshot::Receiver<anyhow::Result<R::Output>>> {
        let (sender, receiver) = oneshot::channel();

        match R::VARIANT {
            RequestableVariant::MultiGet => {
                let cast = request.cast::<SubchunkGetMulti>();
                self.load_multi(cast);
                Ok(receiver)
            },
            RequestableVariant::SingleGet => {
                let cast = request.cast::<SubchunkGetSingle>();
                self.load_single(cast);
                Ok(receiver)
            }
        }
    }

    /// Sets the new value of a game rule and returns the old value if there was one.
    pub fn set_gamerule(&self, val: GameRule) -> Option<GameRule> {
        // Gamerules need rework before finishing this.
        todo!()
    } 

    pub fn gamerule<S: AsRef<str>>(&self, name: S) -> GameRule {
        // Gamerules need rework before finishing this.
        todo!()
    }

    /// Loads a single subchunk.
    fn load_single(&self, request: SubchunkGetSingle) {
        todo!();
    }

    /// Loads multiple subchunks around a center.
    fn load_multi(&self, request: SubchunkGetMulti) {
        todo!();
    }
}