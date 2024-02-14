use std::{
    borrow::{Borrow, Cow},
    fmt::{self, Debug},
    ops::Deref,
};

use crate::{RString, Recycle};

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

enum IntoCowVariant {
    Owned,
    Borrowed,
    Recycled
}

pub trait IntoCowString<'a>: Sized + private::Sealed {
    const VARIANT: IntoCowVariant;

    #[doc(hidden)]
    unsafe fn fsv_cast<T>(self) -> T {
        let cpy = std::mem::transmute_copy::<Self, T>(&self);
        std::mem::forget(self);

        cpy
    }
}

impl IntoCowString<'_> for String {
    const VARIANT: IntoCowVariant = IntoCowVariant::Owned;
}

impl IntoCowString<'_> for RString {
    const VARIANT: IntoCowVariant = IntoCowVariant::Recycled;
}

impl<'a> IntoCowString<'a> for &'a str {
    const VARIANT: IntoCowVariant = IntoCowVariant::Borrowed;
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
            IntoCowVariant::Owned => {
                // SAFETY: This is safe because `V` is guaranteed to be the same type as `String`.
                let owned = Recycle::from(unsafe { val.fsv_cast::<String>() });
                CowString::Owned(owned)
            },
            IntoCowVariant::Borrowed => {
                // SAFETY: This is safe because `V` is guaranteed to be the same type as `&'a str`.
                let borrow = unsafe { val.fsv_cast::<&'a str>() };
                CowString::Borrowed(borrow)
            },
            IntoCowVariant::Recycled => {
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

pub struct CowSliceIter<'data, T> {
    index: usize,
    slice: &'data [T]
}

impl<'data, T> CowSliceIter<'data, T> where [T]: ToOwned {
    pub fn new<'this>(slice: &'this CowSlice<'data, T>) -> CowSliceIter<'this, T> {
        CowSliceIter {
            index: 0,
            slice: slice.borrow()
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
}

pub enum CowSlice<'data, T>
where
    [T]: ToOwned,
{
    Owned(<[T] as ToOwned>::Owned),
    Borrowed(&'data [T]),
}

impl<'data, T> CowSlice<'data, T>
where
    [T]: ToOwned,
{
    pub fn new<V: IntoCowSlice<'data, T>>(val: V) -> CowSlice<'data, T> {
        if V::OWNED {
            // SAFETY: Because V::OWNED is true, `V` is guaranteed to be the same type as the one
            // used in the cast.
            let owned = unsafe { val.fsv_cast::<<[T] as ToOwned>::Owned>() };
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

    pub fn iter<'this>(&'this self) -> CowSliceIter<'this, T> {
        CowSliceIter::new(self)
    }

    /// Indexes into the slice.
    pub fn get(&'data self, idx: usize) -> Option<&'data T> {
        match self {
            CowSlice::Owned(v) => v.borrow().get(idx),
            CowSlice::Borrowed(v) => v.get(idx),
        }
    }
}

impl<'a, 'this, T> IntoIterator for &'this CowSlice<'a, T> where [T]: ToOwned {
    type IntoIter = CowSliceIter<'this, T>;
    type Item = &'this T;

    fn into_iter(self) -> CowSliceIter<'this, T> {
        CowSliceIter::new(self)
    }
}

impl<'a, T> Clone for CowSlice<'a, T>
where
    [T]: ToOwned,
{
    fn clone(&self) -> CowSlice<'a, T> {
        self.to_owned()
    }
}

impl<'a, T> Deref for CowSlice<'a, T>
where
    [T]: ToOwned,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            CowSlice::Borrowed(v) => v,
            CowSlice::Owned(v) => v.borrow(),
        }
    }
}

impl<'a, T> Debug for CowSlice<'a, T>
where
    [T]: ToOwned,
    &'a [T]: Debug,
    <[T] as ToOwned>::Owned: Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            CowSlice::Borrowed(v) => Debug::fmt(v, fmt),
            CowSlice::Owned(v) => Debug::fmt(v, fmt),
        }
    }
}

impl<'a, T> From<&'a [T]> for CowSlice<'a, T>
where
    [T]: ToOwned,
{
    fn from(value: &'a [T]) -> CowSlice<'a, T> {
        CowSlice::Borrowed(value)
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for CowSlice<'a, T>
where
    [T]: ToOwned,
{
    fn from(value: &'a [T; N]) -> CowSlice<'a, T> {
        CowSlice::Borrowed(value)
    }
}

// impl<T> From<<[T] as ToOwned>::Owned> for FastSlice<'static, T>
// where
//     [T]: ToOwned
// {
//     fn from(value: <[T] as ToOwned>::Owned) -> FastSlice<'static, T> {
//         FastSlice::Owned(value)
//     }
// }

impl<'a, T> From<CowSlice<'a, T>> for Cow<'a, [T]>
where
    [T]: ToOwned,
{
    fn from(value: CowSlice<'a, T>) -> Self {
        match value {
            CowSlice::Owned(v) => Cow::Owned(v),
            CowSlice::Borrowed(v) => Cow::Borrowed(v),
        }
    }
}
