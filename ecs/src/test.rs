use std::time::{Duration, Instant};

use crate::{world::{Component, Req, With, World, Without}, Entity};

#[derive(Debug)]
struct Player;

impl Component for Player {}

#[derive(Debug)]
struct Alive;

impl Component for Alive {}

// fn immutable_system(req: Req<&Player, With<Alive>>) {
//     for player in &req {
//         dbg!(player);
//     }
// }

fn entity_dead(dead: Req<Entity, Without<Alive>>) {
    for entity in &dead {
        println!("{:?} is dead", entity.id());
    }
}

fn entity_alive(alive: Req<Entity, With<Alive>>) {
    for entity in &alive {
        println!("{:?} is alive", entity.id());
    }
}

fn entity_all(entities: Req<Entity>) {
    for entity in &entities {
        println!("{:?}", entity.id());
    }
}

fn entity_both(alive: Req<Entity, With<Alive>>, dead: Req<Entity, Without<Alive>>) {
    let mut alive_iter = alive.into_iter();
    let mut dead_iter = dead.into_iter();

    println!("{:?} dead, {:?} alive", dead_iter.count(), alive_iter.count());
}

#[tokio::test(flavor = "multi_thread")]
async fn test() {
    let world = World::new();

    world.spawn((Player, Alive));
    world.spawn(Player);
    world.spawn((Player));
    world.spawn(Player);

    world.system(entity_all);
    world.system(entity_dead);
    world.system(entity_alive);
    world.system(entity_both);

    world.run_all().await;
}