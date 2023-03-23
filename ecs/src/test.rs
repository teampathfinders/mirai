use crate::{request::{Req}, Component, filter::With};

#[derive(Debug)]
struct Player {
    name: &'static str
}

impl Component for Player {}

struct Alive {

}

impl Component for Alive {}

fn system(query: Req<&mut Player, With<Alive>>) {
    for player in &query {
        println!("{player}");
    }
}

#[test]
fn query_test() {

}