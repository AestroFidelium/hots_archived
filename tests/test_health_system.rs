// use heroes_of_the_storm_on_rust::prelude::*;

// #[cfg(test)]
// mod tests {
//     use super::*;

//     fn undo_last_action(entity: Entity, world: &mut World) {
//         let event_to_undo = {
//             if let Ok(mut log) = world.get::<&mut EventLog>(entity) {
//                 log.pop()
//             } else {
//                 None
//             }
//         };

//         if let Some(mut event) = event_to_undo {
//             event.action.undo(entity, world);
//         }
//     }

//     #[test]
//     fn test_regeneration_system() {
//         let mut world = World::new();
//         let player = world.spawn((Health::new(100.0), EventLog::new()));
//         let enemy = world.spawn((Health::new(100.0), EventLog::new()));
//         world.spawn((HealthChange::new(-50.0, Some(enemy), player, None),));
//         health_system(&mut world);
//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 50.0);
//         world.spawn((
//             Timer::new(1.0, true),
//             Action(Box::new(move |_entity, world| {
//                 let regeneration = world.get::<&Health>(player).unwrap().regeneration();
//                 world.spawn((HealthChange::new(regeneration, Some(enemy), player, None),));
//                 TimerActionReturn::Continue
//             })),
//         ));
//         timer_action_system(&mut world, 1.0);
//         health_system(&mut world);
//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 50.110374);
//         timer_action_system(&mut world, 1.0);
//         health_system(&mut world);
//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 50.220748);
//         timer_action_system(&mut world, 1000.0);
//         health_system(&mut world);
//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 100.0);
//     }

//     #[test]
//     fn test_damage_system() {
//         // Здесь будут тесты для системы урона
//         let mut world = World::new();

//         let player = world.spawn((Health::new(100.0), EventLog::new()));
//         let enemy = world.spawn((Health::new(100.0), EventLog::new()));

//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 100.0);

//         world.spawn((HealthChange::new(-10.0, Some(enemy), player, None),));

//         health_system(&mut world);

//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 90.0);

//         health_system(&mut world);
//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 90.0);

//         undo_last_action(player, &mut world);

//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 100.0);

//         world.spawn((HealthChange::new(-50.0, Some(enemy), player, None),));
//         health_system(&mut world);
//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 50.0);

//         world.spawn((HealthChange::new(50.0, Some(enemy), player, None),));
//         health_system(&mut world);
//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 100.0);

//         undo_last_action(player, &mut world);
//         assert_eq!(world.get::<&Health>(player).unwrap().current(), 50.0);
//     }
// }
