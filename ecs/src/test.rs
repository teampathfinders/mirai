use better_any::{Tid, tid};

use crate::{request::{Req}, component::Component, filter::With, system::{System, IntoSystem}, world::World};

#[derive(Debug)]
struct Player {
    name: &'static str
}

tid!(Player);
impl<'t> Component<'t> for Player {}

#[derive(Debug)]
struct Alive; 

tid!(Alive);
impl<'t> Component<'t> for Alive {}

fn system(query: Req<&mut Player, With<Alive>>) {
    for player in &query {
        println!("{player:?}");
    }
}

fn empty_system(_: ()) {
    println!("I am an empty system");
}

#[test]
fn query_test() {
    let mut world = World::new();
    world.spawn(Alive);   
    world.system(empty_system);
    world.execute();

    // let entity = world.spawn((Player { name: "one" }, Alive));
    // dbg!(entity);

    // let entity2 = world.spawn((Player { name: "two" }, Alive));
    // dbg!(entity2);
    // world.despawn(entity2);

    // let entity3 = world.spawn((Player { name: "three" }));
    // dbg!(entity3);

    // let entity4 = world.spawn((Player { name: "four" }, Alive));
    // dbg!(entity4);
    
    // world.system(empty_system);
    // world.execute();
}