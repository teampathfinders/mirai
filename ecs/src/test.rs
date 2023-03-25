use crate::{request::{Req}, component::Component, filter::With, system::{System, IntoSystem}, world::World};

#[derive(Debug)]
struct Player {
    name: &'static str
}

impl Component for Player {}

#[derive(Debug)]
struct Alive; 

impl Component for Alive {}

// fn system(query: Req<&mut Player, With<Alive>>) {
//     for player in &query {
//         println!("{player:?}");
//     }
// }

fn empty_system(req: Req<&Player, With<Alive>>) {
    println!("I am an empty system");
}

#[test]
fn query_test() {
    let mut world = World::new();
    let entity = world.spawn((Player { name: "one" }, Alive));
    let id1 = entity.id();
    dbg!(&id1);

    let entity2 = world.spawn((Player { name: "two" }, Alive));
    dbg!(entity2.id());
    world.despawn(id1);

    let entity3 = world.spawn((Player { name: "three" }));
    dbg!(entity3.id());

    let entity4 = world.spawn((Player { name: "four" }, Alive));
    dbg!(entity4.id());
    
    world.system(empty_system);
    world.execute();
}