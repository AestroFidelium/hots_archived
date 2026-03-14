// use heroes_of_the_storm_on_rust::prelude::*;

// #[cfg(test)]
// mod tests {

//     use super::*;

//     // #[derive(Component)]
//     // struct HelloName(&'static str);

//     // fn hello_world_system(query: Query<&HelloName>) {
//     //     for name in &query {
//     //         println!("Hello, {}!", name.0);
//     //     }
//     // }

//     // #[test]
//     // fn initial_world() {
//     //     let mut world = World::new();
//     //     let mut schedule = Schedule::default();
//     //     schedule.add_systems(hello_world_system);
//     //     world.spawn(HelloName("Alice"));
//     //     world.spawn(HelloName("Bob"));
//     //     schedule.run(&mut world);
//     // }

//     // #[test]
//     // fn test_reversible() {
//     //     struct Health {
//     //         current: Reversible<i32>,
//     //     }

//     //     impl Health {
//     //         pub fn new(current: i32) -> Self {
//     //             Self {
//     //                 current: current.into(),
//     //             }
//     //         }
//     //     }
//     //     let mut health = Health::new(0);

//     //     health.current += 50.into();
//     //     assert_eq!(health.current, 50.into());
//     //     health.current.undo_last();
//     //     assert_eq!(health.current, 0.into());

//     //     health.current.undo_last();
//     //     assert_eq!(health.current, 0.into());
//     // }
// }
