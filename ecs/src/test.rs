use crate::{request::{Request, With}, Component};

struct Player {

}

impl Component for Player {}

struct Alive {

}

impl Component for Alive {}

fn system(query: Request<&mut Player, With<Alive>>) {
    // do something...
}

#[test]
fn query_test() {

}