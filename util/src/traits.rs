use crate::{PVec, BinaryRead, BinaryWrite};
use std::{fmt::Debug, future::Future};

/// Trait that describes an object that can be serialised from raw bytes.
pub trait Serialize {
    /// Estimates the required buffer size to serialize this object.
    /// 
    /// This allows the writer to efficiently allocate enough memory before writing,
    /// preventing reallocations.
    /// By default this function returns `None` which disables the hint.
    fn size_hint(&self) -> Option<usize> { None }

    /// Serializes the object into binary format.
    fn serialize(&self) -> anyhow::Result<PVec> {
        let cap = self.size_hint().unwrap_or(0);
        let mut writer = PVec::alloc_with_capacity(cap);

        self.serialize_into(&mut writer)?;

        Ok(writer)
    }

    /// Serializes the object into binary format into a given writer.
    fn serialize_into<W: BinaryWrite>(&self, writer: &mut W) -> anyhow::Result<()>;
}

/// Trait that describes an object that can be deserialised from raw bytes.
pub trait Deserialize<'a>: Sized {
    /// Deserializes the given buffer, returning the object.
    fn deserialize<R: BinaryRead<'a>>(mut reader: R) -> anyhow::Result<Self> {
        Self::deserialize_from(&mut reader)
    }

    /// Deserializes the given buffer, returning the object.
    /// While [`deserialize`](Self::deserialize) consumes the buffer, this function
    /// modifies the original buffer allowing you to continue where this function left off.
    fn deserialize_from<R: BinaryRead<'a>>(reader: &mut R) -> anyhow::Result<Self>;
}

/// Adds the [`try_expect`](TryExpect::try_expect) function to an object.
pub trait TryExpect {
    /// Output type on successful call.
    type Output;

    /// Similar to the built-in expect function but instead of panicking, it returns an error.
    fn try_expect<E: Debug>(self, message: E) -> anyhow::Result<Self::Output>;
}

impl<T> TryExpect for Option<T> {
    type Output = T;

    fn try_expect<E: Debug>(self, error: E) -> anyhow::Result<Self::Output> {
        match self {
            Some(s) => Ok(s),
            None => {
                anyhow::bail!("{error:?}")
            }
        }
    }
}

pub trait ReserveTo {
    fn reserve_to(&mut self, capacity: usize);
}

impl<T> ReserveTo for Vec<T> {
    fn reserve_to(&mut self, capacity: usize) {
        self.reserve(capacity - self.len());
    }
}

/// Implemented by types that do not shut down instantly and can be joined.
pub trait Joinable {
    /// Asynchronously waits for the service to shut down completely.
    /// 
    /// ## Errors
    /// Usually this method can only be called once on an object.
    /// It is up to the caller to ensure that this is upheld.
    /// 
    /// If the type does not support multiple joins, an error will be returned.
    fn join(&self) -> impl Future<Output = anyhow::Result<()>>;
}   