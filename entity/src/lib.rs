mod component;
mod entity;
mod query;
mod system;
mod world;
mod resource;

use crate::component::Component;
use crate::query::{Query, QueryBundle, With};
use crate::world::World;
use std::fmt::Debug;
use crate::resource::{Res, Resource};

#[derive(Debug)]
struct Timer {
    pub time: u64
}

impl Resource for Timer {}

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

fn empty_system() {
    println!("Hello world!");
}

fn system1(query: Query<&UniqueId>) {
    for id in query {
        println!("{id:?}");
    }
}

#[test]
fn test1() {
    let mut world = World::new();
    // world.system(empty_system);
    // world.system(system1);
    world.spawn(UniqueId(1));

    world.tick();

    // dbg!(world);
}
