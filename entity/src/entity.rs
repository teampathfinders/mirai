#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct EntityId(pub(crate) usize);

pub struct Entities {}

impl Entities {
    pub fn new() -> Self {
        Self {}
    }
}
