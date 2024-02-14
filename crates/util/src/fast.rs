use std::{
    borrow::{Borrow, Cow},
    fmt::{self, Debug},
    ops::Deref,
};

enum FastStringData<'a> {
    Owned(String),
    Borrowed(&'a str),
}

mod private {
    pub trait Sealed {}

    impl Sealed for String {}
    impl Sealed for &str {}
}

pub trait IntoFastString<'a>: Sized + private::Sealed {
    const OWNED: bool;

    #[doc(hidden)]
    unsafe fn fsv_cast<T>(self) -> T {
        let cpy = std::mem::transmute_copy::<Self, T>(&self);
        std::mem::forget(self);

        cpy
    }
}

impl IntoFastString<'_> for String {
    const OWNED: bool = true;
}

impl<'a> IntoFastString<'a> for &'a str {
    const OWNED: bool = false;
}

#[derive(Clone)]
pub enum FastString<'a> {
    Owned(String),
    Borrowed(&'a str),
}

impl<'a> FastString<'a> {
    pub fn new<V: IntoFastString<'a>>(val: V) -> FastString<'a> {
        if V::OWNED {
            // SAFETY: This is safe because `V` is guaranteed to be the same type as `String`.
            let owned = unsafe { val.fsv_cast::<String>() };
            FastString::Owned(owned)
        } else {
            // SAFETY: This is safe because `V` is guaranteed to be the same type as `&'a str`.
            let borrow = unsafe { val.fsv_cast::<&'a str>() };
            FastString::Borrowed(borrow)
        }
    }

    pub fn get(&'a self) -> &'a str {
        match self {
            FastString::Owned(v) => v,
            FastString::Borrowed(v) => v,
        }
    }
}

impl<'a> Deref for FastString<'a> {
    type Target = str;

    fn deref(&self) -> &str {
        self.get()
    }
}

impl<'a> Debug for FastString<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FastString::Borrowed(v) => v.fmt(fmt),
            FastString::Owned(v) => v.fmt(fmt),
        }
    }
}

impl From<String> for FastString<'static> {
    fn from(value: String) -> FastString<'static> {
        FastString::Owned(value)
    }
}

impl<'a> From<&'a str> for FastString<'a> {
    fn from(value: &'a str) -> FastString<'a> {
        FastString::Borrowed(value)
    }
}

impl<'a> From<Cow<'a, str>> for FastString<'a> {
    fn from(value: Cow<'a, str>) -> FastString<'a> {
        match value {
            Cow::Borrowed(v) => FastString::Borrowed(v),
            Cow::Owned(v) => FastString::Owned(v),
        }
    }
}

impl<'a> From<FastString<'a>> for Cow<'a, str> {
    fn from(value: FastString<'a>) -> Cow<'a, str> {
        match value {
            FastString::Borrowed(v) => Cow::Borrowed(v),
            FastString::Owned(v) => Cow::Owned(v),
        }
    }
}

pub trait IntoFastSlice<'a, T>: Sized + private::Sealed {
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

pub enum FastSlice<'a, T>
where
    [T]: ToOwned,
{
    Owned(<[T] as ToOwned>::Owned),
    Borrowed(&'a [T]),
}

impl<'a, T> FastSlice<'a, T>
where
    [T]: ToOwned,
{
    pub fn new<V: IntoFastSlice<'a, T>>(val: V) -> FastSlice<'a, T> {
        if V::OWNED {
            // SAFETY: Because V::OWNED is true, `V` is guaranteed to be the same type as the one
            // used in the cast.
            let owned = unsafe { val.fsv_cast::<<[T] as ToOwned>::Owned>() };
            FastSlice::Owned(owned)
        } else {
            // SAFETY: This is safe because `val` is guaranteed to be of type `&'a [T]`.
            let borrow = unsafe { val.fsv_cast::<&'a [T]>() };
            FastSlice::Borrowed(borrow)
        }
    }

    /// Creates an empty borrowed slice.
    pub const fn empty() -> FastSlice<'static, T> {
        FastSlice::Borrowed(&[])
    }

    /// Gets the inner value of the slice.
    pub fn get(&'a self, idx: usize) -> Option<&'a T> {
        match self {
            FastSlice::Owned(v) => v.borrow().get(idx),
            FastSlice::Borrowed(v) => v.get(idx),
        }
    }
}

impl<'a, T> Clone for FastSlice<'a, T>
where
    [T]: ToOwned,
{
    fn clone(&self) -> FastSlice<'a, T> {
        self.to_owned()
    }
}

impl<'a, T> Deref for FastSlice<'a, T>
where
    [T]: ToOwned,
{
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        match self {
            FastSlice::Borrowed(v) => v,
            FastSlice::Owned(v) => v.borrow(),
        }
    }
}

impl<'a, T> Debug for FastSlice<'a, T>
where
    [T]: ToOwned,
    &'a [T]: Debug,
    <[T] as ToOwned>::Owned: Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::fmt::Result {
        match self {
            FastSlice::Borrowed(v) => Debug::fmt(v, fmt),
            FastSlice::Owned(v) => Debug::fmt(v, fmt),
        }
    }
}

impl<'a, T> From<&'a [T]> for FastSlice<'a, T>
where
    [T]: ToOwned,
{
    fn from(value: &'a [T]) -> FastSlice<'a, T> {
        FastSlice::Borrowed(value)
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for FastSlice<'a, T>
where
    [T]: ToOwned,
{
    fn from(value: &'a [T; N]) -> FastSlice<'a, T> {
        FastSlice::Borrowed(value)
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

impl<'a, T> From<FastSlice<'a, T>> for Cow<'a, [T]>
where
    [T]: ToOwned,
{
    fn from(value: FastSlice<'a, T>) -> Self {
        match value {
            FastSlice::Owned(v) => Cow::Owned(v),
            FastSlice::Borrowed(v) => Cow::Borrowed(v),
        }
    }
}
