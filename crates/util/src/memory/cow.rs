use std::{
    borrow::{Borrow, Cow}, fmt::{self, Debug}, iter::FusedIterator, ops::{Deref, DerefMut}
};

use crate::{RString, Recycled};

enum FastStringData<'a> {
    Owned(String),
    Borrowed(&'a str),
}

mod private {
    use crate::RString;

    pub trait Sealed {}

    impl Sealed for RString {}
    impl Sealed for String {}
    impl Sealed for &str {}
}

/// Which variant is being used to initialise a [`CowString`].
#[doc(hidden)]
pub enum IntoCowStringVariant {
    /// The type used is [`String`].
    Owned,
    /// The type used is `&str`
    Borrowed,
    /// The type used is [`RString`]
    Recycled
}

/// Any items that can directly be converted into a [`CowString`].
pub trait IntoCowString<'a>: Sized + private::Sealed {
    #[doc(hidden)]
    const VARIANT: IntoCowStringVariant;

    #[doc(hidden)]
    unsafe fn fsv_cast<T>(self) -> T {
        let cpy = std::mem::transmute_copy::<Self, T>(&self);
        std::mem::forget(self);

        cpy
    }
}

impl IntoCowString<'_> for String {
    const VARIANT: IntoCowStringVariant = IntoCowStringVariant::Owned;
}

impl IntoCowString<'_> for RString {
    const VARIANT: IntoCowStringVariant = IntoCowStringVariant::Recycled;
}

impl<'a> IntoCowString<'a> for &'a str {
    const VARIANT: IntoCowStringVariant = IntoCowStringVariant::Borrowed;
}

/// A clone-on-write string whose owned data is managed by a global memory pool.
#[derive(Clone)]
pub enum CowString<'a> {
    Owned(RString),
    Borrowed(&'a str),
}

impl<'a> CowString<'a> {
    pub fn new<V: IntoCowString<'a>>(val: V) -> CowString<'a> {
        match V::VARIANT {
            IntoCowStringVariant::Owned => {
                // SAFETY: This is safe because `V` is guaranteed to be the same type as `String`.
                let owned = Recycled::from(unsafe { val.fsv_cast::<String>() });
                CowString::Owned(owned)
            },
            IntoCowStringVariant::Borrowed => {
                // SAFETY: This is safe because `V` is guaranteed to be the same type as `&'a str`.
                let borrow = unsafe { val.fsv_cast::<&'a str>() };
                CowString::Borrowed(borrow)
            },
            IntoCowStringVariant::Recycled => {
                // SAFETY: This is safe because `V` is guaranteed to be the same type as `String`.
                let owned = unsafe { val.fsv_cast::<RString>() };
                CowString::Owned(owned)
            }
        }
    }

    pub fn get(&'a self) -> &'a str {
        match self {
            CowString::Owned(v) => v,
            CowString::Borrowed(v) => v,
        }
    }
}

impl<'a> Deref for CowString<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.get()
    }
}

impl<'a> Debug for CowString<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CowString::Borrowed(v) => v.fmt(fmt),
            CowString::Owned(v) => v.fmt(fmt),
        }
    }
}

impl From<String> for CowString<'static> {
    fn from(value: String) -> CowString<'static> {
        CowString::Owned(RString::from(value))
    }
}

impl<'a> From<&'a str> for CowString<'a> {
    fn from(value: &'a str) -> CowString<'a> {
        CowString::Borrowed(value)
    }
}

impl<'a> From<Cow<'a, str>> for CowString<'a> {
    fn from(value: Cow<'a, str>) -> CowString<'a> {
        match value {
            Cow::Borrowed(v) => CowString::Borrowed(v),
            Cow::Owned(v) => CowString::Owned(RString::from(v)),
        }
    }
}

impl<'a> From<CowString<'a>> for Cow<'a, str> {
    /// ## Warning
    /// 
    /// Converting a `CowString` into a `Cow` will prevent the string from being returned
    /// to the memory pool.
    fn from(value: CowString<'a>) -> Cow<'a, str> {
        match value {
            CowString::Borrowed(v) => Cow::Borrowed(v),
            CowString::Owned(v) => Cow::Owned(v.into_inner()),
        }
    }
}

pub trait IntoCowSlice<'a, T>: Sized + private::Sealed {
    const OWNED: bool;

    /// ## Safety
    ///
    /// This function requires that `Self` and `C` are the exact same type and have the same lifetime.
    #[doc(hidden)]
    unsafe fn fsv_cast<C>(self) -> C {
        let cpy = std::mem::transmute_copy::<Self, C>(&self);
        std::mem::forget(self);

        cpy
    }
}

/// Iterator over elements in a [`CowSlice`].
pub struct CowSliceIter<'data, T> {
    index: usize,
    slice: &'data [T]
}

impl<'data, T> CowSliceIter<'data, T> {
    pub fn new<'this>(slice: &'this CowSlice<'data, T>) -> CowSliceIter<'this, T> {
        CowSliceIter {
            index: 0,
            slice
        }
    }
}

impl<'a, T> Iterator for CowSliceIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        let opt = self.slice.get(self.index);
        self.index += 1;

        opt
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.len()))
    }
}

impl<'a, T> ExactSizeIterator for CowSliceIter<'a, T> {
    fn len(&self) -> usize {
        self.slice.len()
    }
}

impl<'a, T> FusedIterator for CowSliceIter<'a, T> {}

pub enum CowSlice<'data, T> {
    // Owned(<[T] as ToOwned>::Owned),
    Owned(Vec<T>),
    Borrowed(&'data [T]),
}

impl<'data, T> CowSlice<'data, T> {
    pub fn new<V: IntoCowSlice<'data, T>>(val: V) -> CowSlice<'data, T> {
        if V::OWNED {
            // SAFETY: Because V::OWNED is true, `V` is guaranteed to be the same type as the one
            // used in the cast.
            let owned = unsafe { val.fsv_cast::<Vec<T>>() };
            CowSlice::Owned(owned)
        } else {
            // SAFETY: This is safe because `val` is guaranteed to be of type `&'a [T]`.
            let borrow = unsafe { val.fsv_cast::<&'data [T]>() };
            CowSlice::Borrowed(borrow)
        }
    }

    /// Creates an empty borrowed slice.
    pub const fn empty() -> CowSlice<'static, T> {
        CowSlice::Borrowed(&[])
    }

    pub fn iter(&self) -> CowSliceIter<T> {
        CowSliceIter::new(self)
    }

    /// Indexes into the slice.
    pub fn get(&'data self, idx: usize) -> Option<&'data T> {
        match self {
            CowSlice::Owned(v) => v.get(idx),
            CowSlice::Borrowed(v) => v.get(idx),
        }
    }
}

impl<'a, T: Clone> CowSlice<'a, T> {
    /// Converts this slice to its owned variant.
    /// If the slice was already owned, this does nothing.
    pub fn to_owned(&mut self) {
        let CowSlice::Borrowed(borrow) = self else { return };
        let owned = borrow.to_vec();

        *self = CowSlice::Owned(owned);
    }

    pub fn push(&mut self, value: T) {
        self.to_owned();
        match self {
            CowSlice::Owned(owned) => owned.push(value),
            _ => unreachable!()
        }
    }
}

impl<'a, 'this, T: Clone> IntoIterator for &'this CowSlice<'a, T> {
    type IntoIter = CowSliceIter<'this, T>;
    type Item = &'this T;

    fn into_iter(self) -> CowSliceIter<'this, T> {
        CowSliceIter::new(self)
    }
}

impl<'a, T: Clone> Clone for CowSlice<'a, T> {
    fn clone(&self) -> CowSlice<'a, T> {
        match self {
            CowSlice::Borrowed(borrow) => CowSlice::Borrowed(borrow),
            CowSlice::Owned(owned) => CowSlice::Owned(owned.clone())
        }
    }
}

impl<'a, T> Deref for CowSlice<'a, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        match self {
            CowSlice::Borrowed(v) => v,
            CowSlice::Owned(v) => v.borrow(),
        }
    }
}

impl<'a, T: Clone> DerefMut for CowSlice<'a, T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.to_owned();
        match self {
            CowSlice::Owned(vec) => vec.as_mut(),
            _ => unreachable!()
        }
    }
}

impl<'a, T: Debug> Debug for CowSlice<'a, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            CowSlice::Borrowed(v) => Debug::fmt(v, fmt),
            CowSlice::Owned(v) => Debug::fmt(v, fmt),
        }
    }
}

impl<'a, T> From<&'a [T]> for CowSlice<'a, T> {
    fn from(value: &'a [T]) -> CowSlice<'a, T> {
        CowSlice::Borrowed(value)
    }
}

impl<T> From<Vec<T>> for CowSlice<'static, T> {
    fn from(value: Vec<T>) -> CowSlice<'static, T> {
        CowSlice::Owned(value)
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for CowSlice<'a, T> {
    fn from(value: &'a [T; N]) -> CowSlice<'a, T> {
        CowSlice::Borrowed(value)
    }
}

impl<'a, T: Clone> From<CowSlice<'a, T>> for Cow<'a, [T]> {
    fn from(value: CowSlice<'a, T>) -> Cow<'a, [T]> {
        match value {
            CowSlice::Owned(v) => Cow::Owned(v),
            CowSlice::Borrowed(v) => Cow::Borrowed(v),
        }
    }
}
