use crate::world::{Component, Req, With, World};

#[derive(Debug)]
struct Player;

impl Component for Player {}

struct Alive;

impl Component for Alive {}

fn immutable_system(req: Req<&Player, With<Alive>>) {
    for player in &req {
        dbg!(player);
    }
}

#[tokio::test]
async fn test() {
    let mut world = World::new();

    world.spawn((Player, Alive));
    world.system(immutable_system);
    world.run_all();
}