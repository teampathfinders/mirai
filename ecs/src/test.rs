use std::time::{Duration, Instant};

use crate::{world::{Component, Request, With, World, Without}, Entity};

#[derive(Debug)]
struct Player {
    name: &'static str
}

impl Component for Player {}

#[derive(Debug)]
struct Alive;

impl Component for Alive {}

// fn immutable_system(req: Req<&Player, With<Alive>>) {
//     for player in &req {
//         dbg!(player);
//     }
// }

fn entity_dead(dead: Request<Entity, Without<Alive>>) {
    for entity in &dead {
        println!("{:?} is dead", entity.id());
    }
}

fn entity_alive(alive: Request<Entity, With<Alive>>) {
    for entity in &alive {
        println!("{:?} is alive", entity.id());
    }
}

fn entity_all(entities: Request<Entity>) {
    for entity in &entities {
        println!("{:?}", entity.id());
    }
}

fn entity_both(alive: Request<Entity, With<Alive>>, dead: Request<Entity, Without<Alive>>) {
    let mut alive_iter = alive.into_iter();
    let mut dead_iter = dead.into_iter();

    println!("{:?} dead, {:?} alive", dead_iter.count(), alive_iter.count());
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

    world.system(entity_all);
    world.system(entity_dead);
    world.system(entity_alive);
    world.system(entity_both);

    world.run_all().await;
}