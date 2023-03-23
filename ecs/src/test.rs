use crate::{request::{Req}, component::Component, filter::With, system::{System, IntoSystem}, world::World};

#[derive(Debug)]
struct Player {
    name: &'static str
}

impl Component for Player {}

struct Alive; 

impl Component for Alive {}

fn assert_system<Params>(sys: impl IntoSystem<Params>) {
    let sys = sys.into_system();
    
}

fn system(query: Req<&mut Player, With<Alive>>) {
    for player in &query {
        println!("{player:?}");
    }
}

#[test]
fn query_test() {
    let mut world = World::new();
    let entity = world.spawn((Player { name: "player" }, Alive));
    
    world.schedule(system);
}