use crate::{world::{Component, Req, With, World, Without}, Entity};

#[derive(Debug)]
struct Player;

impl Component for Player {}

#[derive(Debug)]
struct Alive;

impl Component for Alive {}

fn immutable_system(req: Req<&Player, With<Alive>>) {
    for player in &req {
        dbg!(player);
    }
}

fn entity_system(req: Req<Entity, Without<Alive>>) {
    for entity in &req {
        entity.despawn();
    }
}

#[tokio::test]
async fn test() {
    let world = World::new();

    world.spawn((Player, Alive));
    world.system(immutable_system);
    world.run_all().await;
}