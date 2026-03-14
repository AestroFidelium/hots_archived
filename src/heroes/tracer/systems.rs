// use crate::prelude::*;

// pub struct TracerQAbility {}

// impl Ability for TracerQAbility {
//     fn activate(&self, world: &mut World, entity: Entity) {
//         let mut query = world.query::<&Position>();
//         if let Ok(pos) = query.get(world, entity) {
//             let distance = 3.0;

//             let mouse = world.get_resource::<Mouse>().unwrap();
//             let camera = world.get_resource::<Camera>().unwrap();
//             let cursor_world = camera
//                 .cursor_to_world(mouse.position[0], mouse.position[1])
//                 .unwrap();

//             let from = Vector3::new(pos.x(), pos.y(), pos.z());
//             let mut dir = Vector3::new(cursor_world.x, from.y, cursor_world.z) - from;

//             if dir.magnitude() > 0.0 {
//                 dir = dir.normalize() * distance;
//             }

//             let to = from + dir;

//             world.entity_mut(entity).remove::<Destination>();

//             world.entity_mut(entity).insert(TracerQAnimation {
//                 from,
//                 to,
//                 step: 0,
//                 total_steps: 10,
//             });
//         }
//     }

//     fn box_clone(&self) -> Box<dyn Ability> {
//         Box::new(TracerQAbility {})
//     }
// }

// pub fn player_ghost(
//     mut query_ghost: Query<
//         (&mut Position, &mut Rotation),
//         (With<PlayerGhost>, Without<Player>, Without<TimeRewind>),
//     >,
//     query_player: Query<
//         (&Position, &Rotation),
//         (Without<PlayerGhost>, With<Player>, Without<TimeRewind>),
//     >,
// ) {
//     if let Ok((player_pos, player_rot)) = query_player.single()
//         && let Ok((mut ghost_pos, mut ghost_rot)) = query_ghost.single_mut()
//     {
//         ghost_pos.set_x(
//             player_pos
//                 .x_ref()
//                 .get_value_seconds_ago(Duration::from_secs(3)),
//         );
//         ghost_pos.set_y(
//             player_pos
//                 .y_ref()
//                 .get_value_seconds_ago(Duration::from_secs(3)),
//         );
//         ghost_pos.set_z(
//             player_pos
//                 .z_ref()
//                 .get_value_seconds_ago(Duration::from_secs(3)),
//         );
//         ghost_rot.yaw_ref_mut().set_value(
//             player_rot
//                 .yaw_ref()
//                 .get_value_seconds_ago(Duration::from_secs(3)),
//         );
//     }
// }

// pub fn tracer_e(
//     mut commands: Commands,
//     query: Query<(Entity, &Position, &Rotation, &Health), With<Player>>,
//     keyboard: Res<Keyboard>,
// ) {
//     if keyboard.is_key_clicked(KeyCode::KeyE)
//         && let Ok((entity, player_pos, rotation, health)) = query.single()
//     {
//         let iter_x = player_pos.x_ref().values_during(Duration::from_secs(3));
//         let iter_y = player_pos.y_ref().values_during(Duration::from_secs(3));
//         let iter_z = player_pos.z_ref().values_during(Duration::from_secs(3));
//         let yaw = rotation.yaw_ref().values_during(Duration::from_secs(3));
//         let iter_health = health.current_ref().values_during(Duration::from_secs(3));
//         let step = iter_x.len();
//         commands.entity(entity).insert(TimeRewind {
//             iter_x,
//             iter_y,
//             iter_z,
//             step,
//             rotate_yaw: yaw,
//             iter_health,
//         });
//     }
// }

// pub fn time_rewind_animation(
//     mut commands: Commands,
//     mut query: Query<
//         (
//             Entity,
//             &mut Position,
//             &mut Rotation,
//             &mut Health,
//             &mut TimeRewind,
//         ),
//         With<Player>,
//     >,
// ) {
//     let speed = 8;

//     if let Ok((entity, mut pos, mut rotation, mut health, mut rewind)) = query.single_mut() {
//         let mut finished = true;

//         for _ in 0..speed {
//             let mut any_popped = false;

//             if let Some(x) = rewind.iter_x.pop() {
//                 pos.x_ref_mut().set_value(x);
//                 any_popped = true;
//             }

//             if let Some(y) = rewind.iter_y.pop() {
//                 pos.y_ref_mut().set_value(y);

//                 any_popped = true;
//             }

//             if let Some(z) = rewind.iter_z.pop() {
//                 pos.z_ref_mut().set_value(z);

//                 any_popped = true;
//             }

//             if let Some(yaw) = rewind.rotate_yaw.pop() {
//                 rotation.yaw_ref_mut().set_value(yaw);

//                 any_popped = true;
//             }

//             if let Some(health_val) = rewind.iter_health.pop() {
//                 health.current_mut().set_value(health_val);

//                 any_popped = true;
//             }

//             if any_popped {
//                 finished = false;
//             } else {
//                 break;
//             }
//         }

//         if finished {
//             commands.entity(entity).remove::<TimeRewind>();
//             commands.entity(entity).remove::<Destination>();

//             if health.current() > 0.0 {
//                 commands.entity(entity).remove::<IsDead>();
//                 commands.entity(entity).insert(IsAlive);
//             }
//         }
//     }
// }

// // pub fn tracer_q(
// //     mut commands: Commands,
// //     mut query: Query<(Entity, &Position), With<TracerQ>>,
// //     keyboard: Res<Keyboard>,
// //     mouse: Res<Mouse>,
// //     camera: Res<Camera>,
// // ) {
// //     if keyboard.is_key_clicked(KeyCode::KeyQ)
// //         && let Ok((entity, pos)) = query.single_mut()
// //         && let Some(cursor_world) = camera.cursor_to_world(mouse.position[0], mouse.position[1])
// //     {
// //         let distance = 3.0;

// //         let from = Vector3::new(pos.x(), pos.y(), pos.z());
// //         let mut dir = Vector3::new(cursor_world.x, from.y, cursor_world.z) - from;

// //         if dir.magnitude() > 0.0 {
// //             dir = dir.normalize() * distance;
// //         }

// //         let to = from + dir;

// //         commands.entity(entity).remove::<Destination>();

// //         commands.entity(entity).insert(TracerQAnimation {
// //             from,
// //             to,
// //             step: 0,
// //             total_steps: 10,
// //         });
// //     }
// // }

// pub fn tracer_q_animation_system(
//     mut commands: Commands,
//     mut query: Query<(Entity, &mut Position, &mut TracerQAnimation)>,
// ) {
//     for (entity, mut pos, mut anim) in query.iter_mut() {
//         anim.step += 1;

//         let t = anim.step as f32 / anim.total_steps as f32;
//         let new_pos = anim.from.lerp(anim.to, t);

//         pos.set_x(new_pos.x);
//         pos.set_y(new_pos.y);
//         pos.set_z(new_pos.z);

//         if anim.step >= anim.total_steps {
//             commands.entity(entity).remove::<TracerQAnimation>();
//         }

//         commands.entity(entity).remove::<Destination>();
//     }
// }

// pub fn tracer_w(
//     mut event: EventWriter<Damage>,
//     mut query: Query<(Entity, &Position, &Team), With<TracerW>>,
//     targets: Query<(Entity, &Position, &Team), (With<Health>, Without<IsDead>)>,
//     keyboard: Res<Keyboard>,
// ) {
//     if keyboard.is_key_clicked(KeyCode::KeyW)
//         && let Ok((entity, pos, team)) = query.single_mut()
//     {
//         let mut target: Option<(Entity, f32)> = None;
//         for (enemy, enemy_pos, enemy_team) in targets.iter() {
//             if enemy == entity || team.player == enemy_team.player {
//                 continue;
//             }

//             let dist = pos.distance_calculate(enemy_pos);

//             if dist <= 3.0 {
//                 if let Some((_, best_dist)) = target {
//                     if dist < best_dist {
//                         target = Some((enemy, dist));
//                     }
//                 } else {
//                     target = Some((enemy, dist));
//                 }
//             }
//         }
//         if let Some((target_entity, _)) = target {
//             event.write(Damage {
//                 attacker: entity,
//                 target: target_entity,
//                 basic_damage: 150.0,
//                 bonus: 0.0,
//                 is_crit: false,
//             });
//         }
//     }
// }

// pub fn tracer_r(
//     mut commands: Commands,
//     mut query: Query<(Entity, &Position), With<TracerR>>,
//     keyboard: Res<Keyboard>,
//     mouse: Res<Mouse>,
//     camera: Res<Camera>,
// ) {
//     if keyboard.is_key_clicked(KeyCode::KeyR)
//         && let Ok((entity, pos)) = query.single_mut()
//         && let Some(cursor_world) = camera.cursor_to_world(mouse.position[0], mouse.position[1])
//     {
//         let distance = 6.0;

//         let from = Vector3::new(pos.x(), pos.y(), pos.z());
//         let mut dir = Vector3::new(cursor_world.x, from.y, cursor_world.z) - from;

//         if dir.magnitude() > 0.0 {
//             dir = dir.normalize() * distance;
//         }

//         let to = from + dir;

//         commands.spawn((MissileBundle::new(
//             RenderBundle {
//                 position: (pos.x(), pos.y(), pos.z()).into(),
//                 size: Size {
//                     x: 3.0.into(),
//                     y: 0.5.into(),
//                     z: 0.5.into(),
//                 },
//                 color: Color {
//                     r: 1.0,
//                     g: 0.0,
//                     b: 0.5,
//                     alpha: 1.0,
//                 },
//                 ..Default::default()
//             },
//             Missile::new(
//                 entity,
//                 MissileTarget::Position(to),
//                 vec![Box::new(TracerREffect {})],
//             ),
//             60.0,
//         ),));
//     }
// }

// pub fn tracer_heal_from_autoattack(
//     query: Query<Entity, With<TracerHealAA>>,
//     mut reader: EventReader<AutoAttackHit>,
//     mut writer: EventWriter<Heal>,
// ) {
//     for event in reader.read() {
//         if query.get(event.caster).is_ok() {
//             writer.write(Heal::new(event.caster, event.caster, 1.0));
//         }
//     }
// }

// pub fn autoattack_ricochet(
//     attacker: Query<(&Position, &Team), With<Ricochet>>,
//     targets: Query<(Entity, &Position, &Team), (With<Health>, Without<IsDead>)>,
//     missiles: Query<&Missile, With<ThisIsAlreadyRicocheted>>,

//     mut event_hit: EventReader<AutoAttackHit>,
//     mut event_launch: EventWriter<AutoAttackLaunch>,
// ) {
//     for event in event_hit.read() {
//         if let AutoAttackSource::Missile(source) = event.source
//             && missiles.get(source).is_ok()
//         {
//             continue;
//         }

//         if let Ok((_attacker_pos, attacker_team)) = attacker.get(event.caster)
//             && let Ok(victim) = targets.get(event.target)
//         {
//             let extra_targets = find_targets_from_position(
//                 victim.1,
//                 attacker_team,
//                 targets.iter(),
//                 7.0,
//                 |enemy, _, _| enemy != event.target,
//                 3,
//             );

//             for target in extra_targets {
//                 event_launch.write(
//                     AutoAttackLaunch::new(
//                         event.caster,
//                         target,
//                         Vector3::new(victim.1.x(), victim.1.y(), victim.1.z()),
//                         AutoAttackSource::JustCreated,
//                     )
//                     .with_insert(ThisIsAlreadyRicocheted),
//                 );
//             }
//         }
//     }
// }

// pub fn vampirism_system(
//     query: Query<&Vampirism>,
//     mut reader: EventReader<Damage>,
//     mut writer: EventWriter<Heal>,
// ) {
//     for event in reader.read() {
//         if let Ok(vampirism) = query.get(event.attacker) {
//             writer.write(Heal::new(
//                 event.attacker,
//                 event.attacker,
//                 (event.basic_damage + event.bonus) * vampirism.strength,
//             ));
//         }
//     }
// }

// pub fn increase_damage_system(query: Query<&IncreaseDamage>, mut mutator: EventMutator<Damage>) {
//     for event in mutator.read() {
//         if let Ok(buff) = query.get(event.attacker) {
//             event.bonus += buff.strength;
//         }
//     }
// }

// pub fn deflect_system(
//     query: Query<Entity, With<IsDeflecting>>,
//     mut missiles: Query<&mut Missile>,
//     mut mutator: EventMutator<MissileHit>,
// ) {
//     for event in mutator.read() {
//         if query.get(event.target).is_ok() {
//             event.is_deleted = true;
//             if let Ok(mut missile) = missiles.get_mut(event.source) {
//                 missile.caster = event.target;
//                 missile.target = MissileTarget::Entity(event.caster);
//             }
//         }
//     }
// }
