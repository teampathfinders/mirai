mod component;
mod entity;
mod system;
mod world;

use crate::component::Component;
use crate::world::World;
use std::fmt::Debug;
use crate::entity::EntityId;

#[derive(Debug)]
struct Health {
    pub value: f32,
}

impl Component for Health {}

#[derive(Debug)]
struct Alive;

impl Component for Alive {}

#[derive(Debug)]
struct UniqueId(usize);

impl Component for UniqueId {}

#[test]
fn test1() {
    let mut world = World::new();
    
}
