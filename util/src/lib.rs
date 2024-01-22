#[macro_use]
mod macros;
#[macro_use]
mod error;
mod u24;

use std::ops::{Deref, DerefMut};

pub use error::*;

pub struct NonCopy<T>(pub T);

impl<T> Deref for NonCopy<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for NonCopy<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

glob_export!(bytes);
glob_export!(traits);
glob_export!(vector);
glob_export!(sync);
