use std::ops::Deref;

use bytes::{BufMut, BytesMut};

use crate::util::WriteExtensions;

/// Type and size independent vector type
#[derive(Debug, Clone)]
pub struct Vector<T, const N: usize> {
    components: [T; N],
}

impl<T: Clone, const N: usize> Vector<T, N> {
    pub fn components(&self) -> [T; N] {
        self.components.clone()
    }
}

impl<T, const N: usize> From<[T; N]> for Vector<T, N> {
    fn from(components: [T; N]) -> Self {
        Self { components }
    }
}

impl<const N: usize> Vector<f32, N> {
    pub fn encode(&self, buffer: &mut BytesMut) {
        for i in 0..N {
            buffer.put_f32(self.components[i]);
        }
    }
}

/// 32-bit float vector with 2 components
pub type Vector2f = Vector<f32, 2>;

/// 32-bit float vector with 3 components
pub type Vector3f = Vector<f32, 3>;

#[derive(Debug, Clone)]
pub struct BlockPosition(i32, u32, i32);

impl BlockPosition {
    pub const fn new(x: i32, y: u32, z: i32) -> Self {
        Self(x, y, z)
    }

    pub fn encode(&self, buffer: &mut BytesMut) {
        buffer.put_var_i32(self.0);
        buffer.put_var_u32(self.1);
        buffer.put_var_i32(self.2);
    }
}
