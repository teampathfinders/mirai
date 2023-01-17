/// Type and size independent vector type
#[derive(Debug, Clone)]
pub struct Vector<T, const N: usize> {
    components: [T; N]
}

/// 32-bit float vector with 2 components
pub type Vector2f = Vector<f32, 2>;

/// 32-bit float vector with 3 components
pub type Vector3f = Vector<f32, 3>;