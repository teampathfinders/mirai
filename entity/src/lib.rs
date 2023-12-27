mod component;
mod entity;
mod system;
mod world;

use crate::component::Component;
use crate::world::World;
use std::fmt::Debug;

#[derive(Debug)]
struct Health {
    pub value: f32,
}

impl Component for Health {}

#[derive(Debug)]
struct Alive;

impl Component for Alive {}

#[test]
fn test1() {
    let mut world = World::new();
    let entity = world.spawn((Alive, Health { value: 1.0 }));

    // let mut mesh = StorageMesh::new();
    // mesh.store(EntityId(0), Health { value: 0.5 });
    // mesh.store(EntityId(1), Health { value: 1.0 });
    //
    // let health = mesh.get_mut::<Health>(EntityId(0)).unwrap();
    // health.value = 1.0;
    //
    // dbg!(mesh);
}
