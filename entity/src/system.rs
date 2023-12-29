use std::fmt::Debug;
use std::marker::PhantomData;
use crate::world::World;

/// Represents a system that can be executed.
pub trait Sys {
    fn call(&self, world: &World);
}

/// This container is needed to constrain the `P` generic.
pub struct SysContainer<P, F: NakedSys<P>> {
    /// The actual function to call when running the system.
    pub(crate) system: F,
    /// Required since `P` is an unused generic parameter.
    /// This parameter is however required to store information about the system
    /// that is needed when executing it.
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

impl<P1, P2, F> Sys for SysContainer<(P1, P2), F>
where
    F: NakedSys<(P1, P2)>,
    (P1, P2): SysParamBundle
    // P1: SysParam, P2: SysParam
{
    fn call(&self, world: &World) {
        self.system.call(world);
    }
}

impl<P1, P2, P3, F> Sys for SysContainer<(P1, P2, P3), F>
where
    F: NakedSys<(P1, P2, P3)>,
    (P1, P2, P3): SysParamBundle
    // P1: SysParam, P2: SysParam, P3: SysParam
{
    fn call(&self, world: &World) {
        self.system.call(world);
    }
}

/// Represents a parameter to a system.
/// This is implemented by several interfaces such as [`Query`] and [`Res`].
/// Anything that implements this trait can be used as a parameter in a system.
///
/// Restricted to [`Sized`] types to be able to use [`Self`] in return types.
pub trait SysParam: Sized {
    /// Indicates whether the parameter requires mutable (and therefore non-parallel)
    /// access to an item.
    const MUTABLE: bool;

    /// Fetches the request object(s) using an immutable reference to the world.
    fn fetch(_world: &World) -> Self {
        panic!("{} does not support immutable fetching", std::any::type_name::<Self>());
    }

    /// Fetches the request object(s) using a mutable reference to the world.
    fn fetch_mut(_world: &mut World) -> Self {
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

impl<P1, P2> SysParamBundle for (P1, P2)
    where P1: SysParam, P2: SysParam
{
    const MUTABLE: bool = P1::MUTABLE || P2::MUTABLE;
}

impl<P1, P2, P3> SysParamBundle for (P1, P2, P3)
    where P1: SysParam, P2: SysParam, P3: SysParam
{
    const MUTABLE: bool = P1::MUTABLE || P2::MUTABLE || P3::MUTABLE;
}

/// Represents a function pointer that is a valid system.
pub trait NakedSys<P>: Sized {
    /// Puts the system into a container and then moves it to the heap to allow for proper storage.
    fn into_container(self) -> Box<dyn Sys>;
    fn call(&self, world: &World);
}

impl<F> NakedSys<()> for F where F: Fn() + 'static {
    fn into_container(self) -> Box<dyn Sys> {
        Box::new(SysContainer { system: self, _marker: PhantomData })
    }

    fn call(&self, _world: &World) {
        self();
    }
}

impl<F, P> NakedSys<P> for F
where
    F: Fn(P) + 'static, P: SysParam + 'static,
{
    fn into_container(self) -> Box<dyn Sys> {
        Box::new(SysContainer { system: self, _marker: PhantomData })
    }

    fn call(&self, world: &World) {
        let p = P::fetch(world);
        self(p);
    }
}

impl<F, P1, P2> NakedSys<(P1, P2)> for F
where
    F: Fn(P1, P2) + 'static, P1: SysParam + 'static, P2: SysParam + 'static
{
    fn into_container(self) -> Box<dyn Sys> {
        Box::new(SysContainer { system: self, _marker: PhantomData })
    }

    fn call(&self, world: &World) {
        let p1 = P1::fetch(world);
        let p2 = P2::fetch(world);

        self(p1, p2);
    }
}

impl<F, P1, P2, P3> NakedSys<(P1, P2, P3)> for F
where
    F: Fn(P1, P2, P3) + 'static,
    P1: SysParam + 'static, P2: SysParam + 'static, P3: SysParam + 'static
{
    fn into_container(self) -> Box<dyn Sys> {
        Box::new(SysContainer { system: self, _marker: PhantomData })
    }

    fn call(&self, world: &World) {
        let p1 = P1::fetch(world);
        let p2 = P2::fetch(world);
        let p3 = P3::fetch(world);

        self(p1, p2, p3);
    }
}

/// Stores and executes the systems.
pub struct Systems {
    /// Systems that have to be executed sequentially because they require mutable access to the world.
    pub(crate) exclusive: Vec<Box<dyn Sys>>,
    /// Systems that only need immutable access and can therefore run in parallel.
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
        self.exclusive.push(system.into_container());

        // println!("is exclusive: {}", P::MUTABLE);
        // todo!()
    }
}
