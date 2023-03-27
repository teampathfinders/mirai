#[cfg(test)]
mod test;

mod component;
mod entity;
mod system;
mod world;
mod request;

mod private {
    pub trait Sealed {}
}