use std::time::Duration;

use parking_lot::RwLock;

use crate::{component::Component, request::{Request, Without, With}, entity::{Entity}, world::World};



#[derive(Debug)]
struct Player {
    name: &'static str
}

impl Component for Player {}

#[derive(Debug)]
struct Alive;

impl Component for Alive {}

fn alive_system(mut req: Request<Entity, With<Alive>>) {
    while let Some(entity) = req.next() {
        dbg!(entity.id());
    }
}

fn player_system(mut req: Request<&'static Player, With<Alive>>) {
    while let Some(player) = req.next() {
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

    world.system(alive_system);
    world.system(player_system);

    world.run_all().await;
}