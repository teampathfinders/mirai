use std::fmt::Debug;
use std::marker::PhantomData;
use crate::world::World;

pub trait Sys {
    fn call(&self, world: &World);
}

pub struct SysContainer<P, F: NakedSys<P>> {
    pub(crate) system: F,
    pub(crate) _marker: PhantomData<P>
}

impl<F> Sys for SysContainer<(), F>
where
    F: NakedSys<()>
{
    fn call(&self, world: &World) {
        self.system.call(world);
    }
}

impl<P, F> Sys for SysContainer<P, F>
where
    F: NakedSys<P>,
    P: SysParam
{
    fn call(&self, world: &World) {
        self.system.call(world);
    }
}

// impl<P1, P2, F> Sys for SysContainer<(P1, P2), F>
// where
//     F: NakedSys<(P1, P2)>,
//     P1: SysParam, P2: SysParam
// {
//     fn call(&self, world: &World) {
//         self.system.call(world);
//     }
// }
//
// impl<P1, P2, P3, F> Sys for SysContainer<(P1, P2, P3), F>
// where
//     F: NakedSys<(P1, P2, P3)>,
//     P1: SysParam, P2: SysParam, P3: SysParam
// {
//     fn call(&self, world: &World) {
//         self.system.call(world);
//     }
// }

/// Represents a parameter to a system.
/// This is implemented by several interfaces such as [`Query`] and [`Res`].
/// Anything that implements this trait can be used as a parameter in a system.
///
/// Restricted to [`Sized`] types to be able to use [`Self`] in return types.
pub trait SysParam: Sized {
    /// Indicates whether the parameter requires mutable (and therefore non-parallel)
    /// access to an item.
    const MUTABLE: bool;

    fn fetch(world: &World) -> Self {
        panic!("{} does not support immutable fetching", std::any::type_name::<Self>());
    }

    fn fetch_mut(world: &mut World) -> Self {
        panic!("{} does not support mutable fetching", std::any::type_name::<Self>());
    }
}

/// Groups multiple system parameters into a single bundle.
/// This is required to restrict the function parameters in systems while also being
/// able to access properties of these parameters.
pub trait SysParamBundle {
    /// Indicates whether a parameter in the bundle requires mutable (and therefore non-parallel)
    /// access to an item.
    const MUTABLE: bool;
}

impl SysParamBundle for () {
    const MUTABLE: bool = false;
}

impl<P: SysParam> SysParamBundle for P {
    const MUTABLE: bool = P::MUTABLE;
}

// impl<'w1, 'w2, P1, P2> SysParamBundle for (P1, P2)
//     where P1: SysParam<'w1>, P2: SysParam<'w2>
// {
//     const MUTABLE: bool = P1::MUTABLE || P2::MUTABLE;
// }
//
// impl<'w1, 'w2, 'w3, P1, P2, P3> SysParamBundle for (P1, P2, P3)
//     where P1: SysParam<'w1>, P2: SysParam<'w2>, P3: SysParam<'w3>
// {
//     const MUTABLE: bool = P1::MUTABLE || P2::MUTABLE || P3::MUTABLE;
// }

pub trait NakedSys<P>: Sized {
    fn into_container(self) -> SysContainer<P, Self> {
        SysContainer { system: self, _marker: PhantomData }
    }

    fn call(&self, world: &World);
}

impl<F> NakedSys<()> for F where F: Fn() {
    fn call(&self, _world: &World) {
        self();
    }
}

impl<F, P> NakedSys<P> for F
where
    F: Fn(P), P: SysParam,
{
    fn call(&self, world: &World) {
        let p = P::fetch(world);
        self(p);
    }
}

// impl<F, P1, P2> NakedSys<(P1, P2)> for F
// where
//     F: Fn(P1, P2), P1: SysParam<'w>, P2: SysParam<'w>
// {
//     fn call(&self, world: &'w World) {
//         let p1 = P1::fetch(world);
//         let p2 = P2::fetch(world);
//
//         self(p1, p2);
//     }
// }

// impl<'w, F, P1, P2, P3> NakedSys<'w, (P1, P2, P3)> for F
// where
//     F: Fn(P1, P2, P3), P1: SysParam<'w>, P2: SysParam<'w>, P3: SysParam<'w>
// {
//     fn call(&self, world: &'w World) {
//         let p1 = P1::fetch(world);
//         let p2 = P2::fetch(world);
//         let p3 = P3::fetch(world);
//
//         self(p1, p2, p3);
//     }
// }

pub struct Systems {
    pub(crate) exclusive: Vec<Box<dyn Sys>>,
    pub(crate) parallel: Vec<Box<dyn Sys>>
}

impl Systems {
    pub fn new() -> Self {
        Systems {
            exclusive: Vec::new(),
            parallel: Vec::new()
        }
    }

    pub fn insert<P, S>(&mut self, system: S)
    where
        P: SysParamBundle + 'static,
        S: NakedSys<P> + 'static,
        SysContainer<P, S>: Sys
    {
        println!("is exclusive: {}", P::MUTABLE);
        // todo!()
    }
}
