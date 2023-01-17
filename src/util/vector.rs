#[derive(Debug, Clone)]
pub struct Vector<T, const N: usize> {
    components: [T; N]
}

pub type Vector2f = Vector<f32, 2>;
pub type Vector3f = Vector<f32, 3>;