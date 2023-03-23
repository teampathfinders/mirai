pub trait Component {

}

impl<T> Component for &T where T: Component {}
impl<T> Component for &mut T where T: Component {}

pub trait ComponentCollection {
    
}

impl<T> ComponentCollection for T where T: Component {
    
}

impl<T0, T1> ComponentCollection for (T0, T1) where T0: Component, T1: Component {
    
}