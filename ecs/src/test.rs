use std::time::Duration;

use parking_lot::RwLock;

use crate::{component::Component, request::{Request, Without, With}, entity::{Entity, EntityMut}, world::World};



#[derive(Debug)]
struct Player {
    name: &'static str
}

impl Component for Player {}

#[derive(Debug)]
struct Alive;

impl Component for Alive {}

fn immutable_system(req: Request<&Player, With<Alive>>) {
    for player in &req {
        dbg!(player);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test() {
    let world = World::new();

    world.spawn((Player {
        name: "Alice"
    }, Alive));

    world.spawn(Player {
        name: "Bob"
    });

    world.spawn((Player {
        name: "Eve"
    }, Alive));

    world.system(immutable_system);

    world.run_all().await;
}