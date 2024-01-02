use core::fmt;

/// A numerical type that can hold 3-byte integers.
///
/// This type is mainly used by Raknet.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct u24([u8; 3]);

impl u24 {
    /// Amount of bits in a `u24`.
    pub const BITS: usize = 24;
    /// Maximum value this type can hold.
    pub const MAX: u24 = u24::from_u32(Self::MAX_U32);
    /// Minimum value this type can hold.
    pub const MIN: u24 = u24::from_u32(0);

    /// Maximum value of a u24 stored as a u32.
    ///
    /// This is used to compute [`u24::MAX`] and check whether a value fits in 24 bits.
    ///
    const MAX_U32: u32 = 16777215;

    /// Converts this value to 3 little endian bytes.
    #[inline]
    pub fn to_le_bytes(&self) -> [u8; 3] {
        let mut out = self.0;
        if cfg!(target_endian = "big") {
            out.reverse();
        }
        out
    }

    /// Converts this value to 3 big endian bytes.
    #[inline]
    pub fn to_be_bytes(&self) -> [u8; 3] {
        let mut out = self.0;
        if cfg!(target_endian = "little") {
            out.reverse();
        }
        out
    }

    /// Creates a `u24` from 3 little endian bytes.
    #[inline]
    pub fn from_le_bytes(mut bytes: [u8; 3]) -> Self {
        if cfg!(target_endian = "big") {
            bytes.reverse();
        }
        Self(bytes)
    }

    /// Creates a `u24` from 3 big endian bytes.
    #[inline]
    pub fn from_be_bytes(mut bytes: [u8; 3]) -> Self {
        if cfg!(target_endian = "little") {
            bytes.reverse();
        }
        Self(bytes)
    }

    /// Converts a `u32` to a `u24`.
    ///
    /// # Panic
    /// This function panics if the value does not fit in 24 bits.
    /// If this is not desired, use the [`TryFrom`] implementation instead.
    #[inline]
    pub(crate) const fn from_u32(v: u32) -> Self {
        debug_assert!(v <= Self::MAX_U32);

        let b = v.to_ne_bytes();
        Self([b[0], b[1], b[2]])
    }

    /// Converts a `u24` to a `u32`.
    #[inline]
    pub(crate) const fn to_u32(self) -> u32 {
        if cfg!(target_endian = "big") {
            u32::from_be_bytes([self.0[0], self.0[1], self.0[2], 0])
        } else {
            u32::from_le_bytes([0, self.0[0], self.0[1], self.0[2]])
        }
    }
}

impl fmt::Debug for u24 {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{:?}", self.to_u32())
    }
}

impl TryFrom<u32> for u24 {
    type Error = anyhow::Error;

    fn try_from(value: u32) -> anyhow::Result<Self> {
        if value > Self::MAX_U32 {
            bail!(Other, "value {value} is too big to fit in u24");
        }

        let bytes = value.to_ne_bytes();
        Ok(Self([bytes[0], bytes[1], bytes[2]]))
    }
}

impl From<u24> for u32 {
    fn from(value: u24) -> Self {
        let b = value.0;
        (b[0] as u32) | ((b[1] as u32) << 8) | ((b[2] as u32) << 16)
    }
}
