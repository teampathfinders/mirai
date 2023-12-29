use std::fmt::Debug;
use std::marker::PhantomData;
use crate::component::Components;

/// Represents a system that can be executed.
pub trait Sys<'w> {
    fn call(&self, components: &'w Components);
}

/// This container is needed to constrain the `P` generic.
pub struct SysContainer<'w, P, F: NakedSys<'w, P>> {
    /// The actual function to call when running the system.
    pub(crate) system: F,
    /// Required since `P` is an unused generic parameter.
    /// This parameter is however required to store information about the system
    /// that is needed when executing it.
    pub(crate) _marker: PhantomData<&'w P>
}

impl<'w, F> Sys<'w> for SysContainer<'w, (), F>
where
    F: NakedSys<'w, ()>
{
    fn call(&self, components: &'w Components) {
        self.system.call(components);
    }
}

impl<'w, P, F> Sys<'w> for SysContainer<'w, P, F>
where
    F: NakedSys<'w, P>,
    P: SysParam<'w>
{
    fn call(&self, components: &'w Components) {
        self.system.call(components);
    }
}

impl<'w, P1, P2, F> Sys<'w> for SysContainer<'w, (P1, P2), F>
where
    F: NakedSys<'w, (P1, P2)>,
    (P1, P2): SysParamBundle
    // P1: SysParam, P2: SysParam
{
    fn call(&self, components: &'w Components) {
        self.system.call(components);
    }
}

impl<'w, P1, P2, P3, F> Sys<'w> for SysContainer<'w, (P1, P2, P3), F>
where
    F: NakedSys<'w, (P1, P2, P3)>,
    (P1, P2, P3): SysParamBundle
    // P1: SysParam, P2: SysParam, P3: SysParam
{
    fn call(&self, components: &'w Components) {
        self.system.call(components);
    }
}

/// Represents a parameter to a system.
/// This is implemented by several interfaces such as [`Query`] and [`Res`].
/// Anything that implements this trait can be used as a parameter in a system.
///
/// Restricted to [`Sized`] types to be able to use [`Self`] in return types.
pub trait SysParam<'w>: Sized {
    /// Indicates whether the parameter requires mutable (and therefore non-parallel)
    /// access to an item.
    const MUTABLE: bool;

    /// Fetches the request object(s) using an immutable reference to the world.
    fn fetch(_components: &'w Components) -> Self {
        panic!("{} does not support immutable fetching", std::any::type_name::<Self>());
    }

    /// Fetches the request object(s) using a mutable reference to the world.
    fn fetch_mut(_components: &'w mut Components) -> Self {
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

impl<'w, P: SysParam<'w>> SysParamBundle for P {
    const MUTABLE: bool = P::MUTABLE;
}

impl<'w1, 'w2, P1, P2> SysParamBundle for (P1, P2)
    where P1: SysParam<'w1>, P2: SysParam<'w2>
{
    const MUTABLE: bool = P1::MUTABLE || P2::MUTABLE;
}

impl<'w1, 'w2, 'w3, P1, P2, P3> SysParamBundle for (P1, P2, P3)
    where P1: SysParam<'w1>, P2: SysParam<'w2>, P3: SysParam<'w3>
{
    const MUTABLE: bool = P1::MUTABLE || P2::MUTABLE || P3::MUTABLE;
}

/// Represents a function pointer that is a valid system.
pub trait NakedSys<'w, P>: Sized {
    /// Puts the system into a container and then moves it to the heap to allow for proper storage.
    fn into_container(self) -> Box<dyn Sys<'w> + 'w>;
    fn call(&self, components: &'w Components);
}

impl<'w, F> NakedSys<'w, ()> for F where F: Fn() + 'static {
    fn into_container(self) -> Box<dyn Sys<'w> + 'w> {
        Box::new(SysContainer { system: self, _marker: PhantomData })
    }

    fn call(&self, _components: &'w Components) {
        self();
    }
}

impl<'w, F, P> NakedSys<'w, P> for F
where
    F: Fn(P) + 'static, P: SysParam<'w> + 'static,
{
    fn into_container(self) -> Box<dyn Sys<'w> + 'w> {
        Box::new(SysContainer { system: self, _marker: PhantomData })
    }

    fn call(&self, components: &'w Components) {
        let p = P::fetch(components);
        self(p);
    }
}

impl<'w, F, P1, P2> NakedSys<'w, (P1, P2)> for F
where
    F: Fn(P1, P2) + 'static, P1: SysParam<'w> + 'static, P2: SysParam<'w> + 'static
{
    fn into_container(self) -> Box<dyn Sys<'w> + 'w> {
        Box::new(SysContainer { system: self, _marker: PhantomData })
    }

    fn call(&self, components: &'w Components) {
        let p1 = P1::fetch(components);
        let p2 = P2::fetch(components);

        self(p1, p2);
    }
}

// impl<'w1, 'w2, 'w3, F, P1, P2, P3> NakedSys<'w1, (P1, P2, P3)> for F
// where
//     F: Fn(P1, P2, P3) + 'static,
//     P1: SysParam<'w1> + 'static, P2: SysParam<'w2> + 'static, P3: SysParam<'w3> + 'static
// {
//     fn into_container(self) -> Box<dyn Sys<'w1> + 'w1> {
//         Box::new(SysContainer { system: self, _marker: PhantomData })
//     }
//
//     fn call(&self, components: &Components) {
//         let p1 = P1::fetch(components);
//         let p2 = P2::fetch(components);
//         let p3 = P3::fetch(components);
//
//         self(p1, p2, p3);
//     }
// }

/// Stores and executes the systems.
pub(crate) struct Systems<'w> {
    /// Systems that have to be executed sequentially because they require mutable access to the world.
    exclusive: Vec<Box<dyn Sys<'w> + 'w>>,
    /// Systems that only need immutable access and can therefore run in parallel.
    parallel: Vec<Box<dyn Sys<'w> + 'w>>
}

impl<'w> Systems<'w> {
    pub fn new() -> Self {
        Systems {
            exclusive: Vec::new(),
            parallel: Vec::new()
        }
    }

    pub fn insert<P, S>(&mut self, system: S)
    where
        P: SysParamBundle + 'w,
        S: NakedSys<'w, P>,
        SysContainer<'w, P, S>: Sys<'w>
    {
        self.exclusive.push(system.into_container());
    }

    pub fn tick(&mut self, components: &'w Components) {
        for system in &self.exclusive {
            system.call(components)
        }
    }
}
