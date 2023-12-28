mod component;
mod entity;
mod query;
mod system;
mod world;

use crate::component::Component;
use crate::entity::EntityId;
use crate::query::{Query, QueryBundle, With};
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

#[derive(Debug)]
struct UniqueId(usize);

impl Component for UniqueId {}

fn query(query: Query<(&UniqueId, &mut Health), With<Alive>>) {
    for (id, health) in query {
        dbg!(id);
        dbg!(health);
    }
}

#[test]
fn test1() {
    let mut world = World::new();
    for i in 0..10 {
        world.spawn((UniqueId(i), Alive, Health { value: i as f32 }));
    }
    world.get_mut(EntityId(2)).unwrap().despawn();

    println!("{}", world.entities.is_spawned(EntityId(2)));

    // dbg!(world);
}
