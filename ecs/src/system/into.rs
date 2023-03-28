use std::sync::Arc;

use super::{param::Param, systems::Sys, parallel::ParallelSystem, sequential::SequentialSystem};

pub trait IntoSys<S, Params> {
    fn into_boxed(self) -> Arc<dyn Sys + Send + Sync>;
}

impl<S, P> IntoSys<S, P> for S 
where
    S: Fn(P) + Send + Sync + 'static,
    P: Param + Send + Sync + 'static
{
    fn into_boxed(self) -> Arc<dyn Sys + Send + Sync> {
        if P::READONLY {
            Arc::new(ParallelSystem::new(self))
        } else {
            Arc::new(SequentialSystem::new(self))
        }
    }
}