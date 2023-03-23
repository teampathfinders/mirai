use crate::{request::{Req}, Component, filter::With, system::{System, IntoSystem}};

#[derive(Debug)]
struct Player {
    name: &'static str
}

impl Component for Player {}

struct Alive {

}

impl Component for Alive {}

fn assert_system<F, P, S: IntoSystem<F, P>>(sys: S) {
    let sys = sys.into_system();
    sys.print();
}

fn system(query: Req<&mut Player, With<Alive>>) {
    for player in &query {
        println!("{player:?}");
    }
}

#[test]
fn query_test() {
    assert_system(system);
}