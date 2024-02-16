use std::{any::TypeId, sync::{Arc, OnceLock, Weak}};

use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
use util::Joinable;

use crate::instance::Instance;

use super::{gamerule, GameRule, GameRuleValue, TntExplodes};

pub struct Service {
    instance_token: CancellationToken,
    shutdown_token: CancellationToken,
    instance: OnceLock<Weak<Instance>>,

    gamerules: DashMap<TypeId, GameRuleValue>
}

impl Service {
    pub(crate) fn new(instance_token: CancellationToken) -> Arc<Service> {
        Arc::new(Service {
            instance_token,
            shutdown_token: CancellationToken::new(),
            instance: OnceLock::new(),

            gamerules: DashMap::new()
        })
    }

    /// Sets the parent instance of this service.
    pub(crate) fn set_instance(&self, instance: &Arc<Instance>) -> anyhow::Result<()> {
        self.set_gamerule::<TntExplodes>(true);

        self.instance
            .set(Arc::downgrade(instance))
            .map_err(|_| anyhow::anyhow!("Level service instance was already set"))
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
    /// See [`GameRule`] for defining your own custom gamerules.
    pub fn set_gamerule<R: GameRule>(&self, value: R::Value) -> R::Value {
        let value = R::to_value(value);
        let old = self.gamerules.insert(TypeId::of::<R>(), value);
        
        let Some(old) = old else {
            return R::Value::default()
        };

        R::from_value(old)
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
    /// See [`GameRule`] for defining your own custom gamerules.
    pub fn gamerule<R: GameRule>(&self) -> R::Value {
        let Some(kv) = self.gamerules.get(&TypeId::of::<R>()) else {
            return R::Value::default()
        };
        
        R::from_value(*kv.value())
    }
}

impl Joinable for Service {
    async fn join(&self) -> anyhow::Result<()> {
        self.shutdown_token.cancelled().await;
        Ok(())
    }
}