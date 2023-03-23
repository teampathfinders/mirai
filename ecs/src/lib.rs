#[cfg(test)]
mod test;

mod request;
mod filter;
mod system;

pub trait Component {

}

impl<T> Component for &T where T: Component {}
impl<T> Component for &mut T where T: Component {}