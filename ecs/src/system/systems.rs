use std::{marker::PhantomData, sync::Arc, mem};

use parking_lot::RwLock;

use crate::{world::WorldState, request::{Requestable, Filters, Request}};

use super::{param::ParamSet, into::IntoSys};

pub trait Sys {
    fn call(&self, state: Arc<RwLock<WorldState>>) {
        unimplemented!()
    }

    fn call_mut(&self, state: &mut WorldState) {
        unimplemented!()
    }
}

#[derive(Default)]
pub struct Systems {
    parallel: RwLock<Vec<Arc<dyn Sys + Send + Sync>>>,
    sequential: RwLock<Vec<Arc<dyn Sys + Send + Sync>>>
}

impl Systems {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<S, P: ParamSet>(&self, system: impl IntoSys<S, P>) {
        if P::READONLY {
            self.parallel.write().push(system.into_boxed());
        } else {
            self.sequential.write().push(system.into_boxed());
            todo!();
        }
    }

    pub async fn run_all(&self, state: Arc<RwLock<WorldState>>) {
        let lock = self.parallel.read();
        for system in &*lock {
            system.call(state.clone());
        }

        // Wait for all systems to finish running
        // while task_set.join_next().await.is_some() {}
    }
}