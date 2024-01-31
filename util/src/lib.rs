#[macro_use]
mod macros;
#[macro_use]
mod error;
mod u24;

use std::{fmt, ops::{Deref, DerefMut}, sync::atomic::{AtomicU64, Ordering}};

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

pub trait Zeroize {
    fn zeroize(&mut self);
}

macro_rules! impl_zeroize_integers {
    ($($ty: ty),+) => {
        $(
            impl Zeroize for $ty {
                #[inline]
                fn zeroize(&mut self) {
                    unsafe {
                        std::ptr::write_volatile(self as *mut $ty, 0);
                    }
                    std::sync::atomic::compiler_fence(Ordering::SeqCst);
                }
            }
        )+
    }
}

impl_zeroize_integers!(u8, i8, u16, i16, i32, u32, i64, u64, i128, u128);

impl<T: Zeroize, const N: usize> Zeroize for [T; N] {
    #[inline]
    fn zeroize(&mut self) {
        self.iter_mut().for_each(|elem| elem.zeroize());
    }
}

impl Zeroize for AtomicU64 {
    #[inline]
    fn zeroize(&mut self) {
        self.store(0, Ordering::SeqCst);
    }
}

/// Allows access to a value inside of a [`Secret`].
pub trait ExposeSecret<T> {
    /// Exposes the value inside of the secret.
    /// 
    /// Warning: This means the value inside of the secret will be unprotected while
    /// it is exposed. The value is a secret for a reason, so be careful with what you do with it.
    fn expose(&self) -> &T;

    /// Exposes the value inside of the secret.
    /// 
    /// Warning: This means the value inside of the secret will be unprotected while
    /// it is exposed. The value is a secret for a reason, so be careful with what you do with it.
    fn expose_mut(&mut self) -> &mut T;
}

/// Overrides display and debug implementations to prevent accidental leakage of secure values
/// in logs. 
/// 
/// This also overwrites the value on drop to make sure it does not last in memory.
/// Zeroizing is done on a best-effort basis.
pub struct Secret<T: Zeroize> {
    value: T
}

impl<T: Zeroize> ExposeSecret<T> for Secret<T> {
    #[inline]
    fn expose(&self) -> &T {
        &self.value
    }

    #[inline]
    fn expose_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Zeroize> Secret<T> {
    /// Creates a new secret.
    #[inline]
    pub const fn new(value: T) -> Secret<T> {
        Secret { value }
    }
}

impl<T: Zeroize> Drop for Secret<T> {
    fn drop(&mut self) {
        self.value.zeroize();
    }
}

impl<T: Zeroize> fmt::Debug for Secret<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "[secret]")
    }
}

impl<T: Zeroize> fmt::Display for Secret<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "[secret]")
    }
}

impl<T: Zeroize + Default> Default for Secret<T> {
    fn default() -> Secret<T> {
        Secret { value: T::default() }
    }
}

impl<T: Zeroize> From<T> for Secret<T> {
    fn from(value: T) -> Secret<T> {
        Secret { value }
    }
}

glob_export!(bytes);
glob_export!(traits);
glob_export!(vector);
glob_export!(sync);
