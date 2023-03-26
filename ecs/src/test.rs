use crate::world::{Component, Req, With};

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

}