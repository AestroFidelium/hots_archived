// use crate::prelude::*;

// /// Move entities towards their `Destination` using `MovementSpeed`.
// //
// /// - Query: (Entity, &mut Position, &mut Rotation, &Destination, &MovementSpeed) with `CanMoving`.
// /// - Side-effects: updates `Position` and `Rotation`; inserts/removes `IsMoving` and `IsRotating` tags via `Commands`.
// /// - Notes: uses a fixed delta of 0.016 for the movement step. Keep ordering so movement runs before render collection.
// pub fn movement_system(
//     mut commands: Commands,
//     mut query: Query<
//         (
//             Entity,
//             &mut Position,
//             &mut Rotation,
//             &Destination,
//             &MovementSpeed,
//         ),
//         With<CanMoving>,
//     >,
// ) {
//     for (entity, mut pos, mut rot, dest, speed) in &mut query {
//         let current = Vector3::new(pos.x(), pos.y(), pos.z());
//         let target = Vector3::new(dest.x(), dest.y(), dest.z());

//         let direction = target - current;
//         let distance = direction.magnitude();

//         if distance < 0.01 {
//             commands.entity(entity).remove::<IsMoving>();
//             commands.entity(entity).remove::<Destination>();
//             commands.entity(entity).insert(IsStanding);

//             continue;
//         }

//         commands.entity(entity).insert(IsMoving);
//         commands.entity(entity).remove::<IsStanding>();

//         let move_step = speed.current() * 0.016;
//         let movement = direction.normalize() * move_step.min(distance);

//         *pos.x_ref_mut() += movement.x;
//         *pos.y_ref_mut() += movement.y;
//         *pos.z_ref_mut() += movement.z;

//         let dir_normalized = direction.normalize();
//         let yaw_angle = dir_normalized.z.atan2(dir_normalized.x);

//         if yaw_angle <= 0.0 {
//             commands.entity(entity).remove::<IsRotating>();
//         }

//         commands.entity(entity).insert(IsRotating);

//         rot.yaw_ref_mut().set_value(-yaw_angle);
//     }
// }

// pub fn player_right_click(
//     mut commands: Commands,
//     query: Query<Entity, With<Player>>,
//     camera: Res<Camera>,
//     mouse: Res<Mouse>,
// ) {
//     if mouse.is_button_hold(MouseButton::Right)
//         && let Ok(entity) = query.single()
//         && let Some(position) = camera.cursor_to_world(mouse.position[0], mouse.position[1])
//     {
//         commands
//             .entity(entity)
//             .insert(Destination::new(position.x, 1.0, position.z));
//     }
// }

// pub fn player_camera_backspace(
//     query: Query<&Position, With<Player>>,
//     mut camera: ResMut<Camera>,
//     keyboard: Res<Keyboard>,
// ) {
//     if keyboard.is_key_hold(KeyCode::Space)
//         && let Ok(position) = query.single()
//     {
//         let offset = Vector3::new(0.0, 15.0, 8.0);
//         camera.position = Point3::new(position.x(), position.y(), position.z()) + offset;
//         camera.target = Point3::new(position.x(), position.y(), position.z());
//     }
// }

// pub fn render_system(
//     query: Query<(&Position, &Rotation, Option<&Color>, Option<&Size>), With<Renderable>>,
//     mut models: ResMut<RenderModels>,
// ) {
//     for (pos, rot, col, size) in query {
//         let color_arr = col.map(|c| c.get_color()).unwrap_or([1.0, 1.0, 1.0, 1.0]);
//         let size = size
//             .map(|c| [c.x(), c.y(), c.z()])
//             .unwrap_or([1.0, 1.0, 1.0]);

//         models.0.push(
//             ModelData::color(color_arr)
//                 .translate([pos.x(), pos.y(), pos.z()])
//                 .rotate_yaw(rot.yaw())
//                 .rotate_pitch(rot.pitch())
//                 .rotate_roll(rot.roll())
//                 .scale_xyz(size[0], size[1], size[2]),
//         );
//     }
// }

// pub fn render_clear_system(
//     mut world: ResMut<ResourceWorldText>,
//     mut screen: ResMut<ResourceScreenText>,
//     mut models: ResMut<RenderModels>,
// ) {
//     models.0.clear();
//     world.texts.clear();
//     screen.texts.clear();
// }

// pub fn health_render_system(
//     query: Query<(&Position, &Health), With<Renderable>>,
//     player_health: Query<&Health, (With<Renderable>, With<Player>)>,
//     mut screen: ResMut<ResourceScreenText>,
//     mut models: ResMut<RenderModels>,
//     camera: Res<Camera>,
// ) {
//     let view = camera.view_matrix();
//     let mut rot_only = view;
//     rot_only.w = cgmath::Vector4::unit_w();
//     rot_only = rot_only.transpose();

//     for (pos, health) in query {
//         let model = Matrix4::from_translation([pos.x(), pos.y() + 3.0, pos.z()].into())
//             * rot_only
//             * Matrix4::from_nonuniform_scale(1.0, 1.0, 0.005);

//         let t = (health.current() / health.max()).clamp(0.0, 1.0);
//         let r = 1.0 - t * t;
//         let g = t * t;
//         let b = 0.0;
//         let color = [r, g, b, 1.0];

//         if t < 1.0 {
//             models.0.push(
//                 ModelData::color([0.25, 0.25, 0.25, 1.0])
//                     .with_matrix(model)
//                     .scale_xyz(1.0, 0.3, 0.1),
//             );
//         }

//         let health_model = model * Matrix4::from_translation([-(1.0 - t) * 0.5, 0.0, 0.0].into());

//         models.0.push(
//             ModelData::color(color)
//                 .with_matrix(health_model)
//                 .scale_xyz(t, 0.3, 0.1), // ← вот тут X = t
//         );
//     }

//     if let Ok(player) = player_health.single() {
//         screen.texts.push(Text {
//             text: format!("HP: {}", player.current()),
//             position: Vector3 {
//                 x: 150.0,
//                 y: 150.0,
//                 z: 0.0,
//             },
//             scale: 1.0,
//             ..Default::default()
//         });
//     }
// }

// pub fn convenor_to_world(
//     query: Query<(&WorldText, &Position, &Color)>,
//     mut render: ResMut<ResourceWorldText>,
// ) {
//     for (text, position, color) in query {
//         render.texts.push(Text {
//             text: text.text.clone(),
//             color: *color,
//             scale: text.scale,
//             position: position.into(),
//         });
//     }
// }

// pub fn display_health_output(
//     mut commands: Commands,
//     mut texts: Query<(
//         Entity,
//         &mut WorldText,
//         &mut Lifespan,
//         &mut HealthChangeText,
//         &mut Color,
//         &mut Position,
//         &mut Destination,
//     )>,
//     query: Query<(&Position, &Health), Without<HealthChangeText>>,
//     mut event_damage: EventReader<Damage>,
//     mut event_heal: EventReader<Heal>,
// ) {
//     // общий хелпер
//     let mut spawn_or_update = |owner: Entity, delta: f32, pos: &Position| {
//         if let Some((
//             _id,
//             mut wt,
//             mut lifespan,
//             mut tag,
//             mut color,
//             mut position,
//             mut destination,
//         )) = texts
//             .iter_mut()
//             .find(|(_, _, _, tag, _, _, _)| tag.owner == owner)
//         {
//             tag.value += delta;
//             wt.text = format!(
//                 "{:.0}",
//                 if tag.value > 0.0 {
//                     tag.value
//                 } else {
//                     -tag.value
//                 }
//             );

//             *color = Color::from_hex(if tag.value > 0.0 {
//                 "#47F04F"
//             } else {
//                 "#F0474F"
//             })
//             .unwrap();

//             *position = pos.values_clone().with_y(pos.y() + 1.0);
//             *destination = Destination::new(pos.x(), pos.y() + 20.0, pos.z());

//             lifespan.spawned = Instant::now();
//         } else {
//             commands.spawn((
//                 WorldText {
//                     text: format!("{:.0}", if delta > 0.0 { delta } else { -delta }),
//                     scale: 0.01,
//                 },
//                 HealthChangeText {
//                     owner,
//                     value: delta,
//                 },
//                 Lifespan::new(Duration::from_millis(500)),
//                 Color::from_hex(if delta > 0.0 { "#47F04F" } else { "#F0474F" }).unwrap(),
//                 pos.values_clone().with_y(pos.y() + 1.0),
//                 Destination::new(pos.x(), pos.y() + 20.0, pos.z()),
//                 MovementSpeed::new(1.0),
//                 Rotation::default(),
//                 CanMoving,
//             ));
//         }
//     };

//     for damage in event_damage.read() {
//         if let Ok((pos, _)) = query.get(damage.target) {
//             let value = damage.basic_damage + damage.bonus;
//             spawn_or_update(
//                 damage.target,
//                 -value, // урон отрицательный
//                 pos,
//             );
//         }
//     }

//     for heal in event_heal.read() {
//         if let Ok((pos, health)) = query.get(heal.target) {
//             if health.current() == health.max() {
//                 continue;
//             }
//             let value = (heal.basic_heal + heal.bonus).min(health.max() - health.current());
//             spawn_or_update(
//                 heal.target,
//                 value, // хил положительный
//                 pos,
//             );
//         }
//     }
// }

// pub fn damage_system(
//     mut commands: Commands,
//     mut reader: EventReader<Damage>,
//     mut query: Query<(Entity, &mut Health), Without<IsDead>>,
// ) {
//     for damage in reader.read() {
//         if let Ok((entity, mut health)) = query.get_mut(damage.target) {
//             let value = damage.basic_damage + damage.bonus;
//             *health.current_mut() -= value;

//             if health.current() <= 0.0 {
//                 commands.entity(entity).insert(IsDead);
//                 commands.entity(entity).remove::<IsAlive>();
//             }
//         }
//     }
// }

// pub fn heal_system(mut reader: EventReader<Heal>, mut query: Query<&mut Health, Without<IsDead>>) {
//     for heal in reader.read() {
//         if let Ok(mut health) = query.get_mut(heal.target) {
//             if health.current() == health.max() {
//                 continue;
//             }

//             let value = (heal.basic_heal + heal.bonus).min(health.max() - health.current());
//             *health.current_mut() += value;
//         }
//     }
// }

// /// Вызывается когда противник в радиусе автоатаки, запускает систему автоатак.
// pub fn autoattack_enemy_in_range_distance_system(
//     mut commands: Commands,
//     mut attackers: Query<
//         (
//             Entity,
//             &Position,
//             &AutoAttack,
//             &Team,
//             Option<&mut AttackTimer>,
//         ),
//         (With<IsStanding>, Without<IsDead>),
//     >,
//     targets: Query<(Entity, &Position, &Team), (With<Health>, Without<IsDead>)>,
//     mut event_start: EventWriter<AutoAttackStart>,
// ) {
//     attackers.iter_mut().for_each(
//         |(attacker, attacker_pos, autoattack, attacker_team, timer_opt)| {
//             let mut can_attack = true;
//             let now = Instant::now();
//             if let Some(mut timer) = timer_opt {
//                 let elapsed = now.duration_since(timer.last_attack).as_secs_f32();
//                 if elapsed < 1.0 / autoattack.speed.value() {
//                     can_attack = false;
//                 } else {
//                     timer.last_attack = now;
//                 }
//             } else {
//                 commands
//                     .entity(attacker)
//                     .insert(AttackTimer { last_attack: now });
//             }

//             if !can_attack {
//                 commands.entity(attacker).remove::<IsAutoAttacking>();
//                 return;
//             }

//             commands.entity(attacker).insert(IsAutoAttacking);

//             let attack_range = match autoattack.range {
//                 AutoAttackRange::Melee => 2.0,
//                 AutoAttackRange::Range(r) => r as f32,
//             };

//             let mut target: Option<(Entity, f32)> = None;
//             for (enemy, enemy_pos, enemy_team) in targets.iter() {
//                 if enemy == attacker || attacker_team.player == enemy_team.player {
//                     continue;
//                 }

//                 let dist = attacker_pos.distance_calculate(enemy_pos);

//                 if dist <= attack_range {
//                     if let Some((_, best_dist)) = target {
//                         if dist < best_dist {
//                             target = Some((enemy, dist));
//                         }
//                     } else {
//                         target = Some((enemy, dist));
//                     }
//                 }
//             }

//             if let Some((victim, _)) = target {
//                 event_start.write(AutoAttackStart::new(
//                     attacker,
//                     victim,
//                     Vector3::new(attacker_pos.x(), attacker_pos.y(), attacker_pos.z()),
//                     AutoAttackSource::JustCreated,
//                     AttackType::Original,
//                 ));

//                 commands.entity(victim).insert(IsTargetedByAutoAttack);
//             }
//         },
//     );
// }

// pub fn autoattack_start_to_launch_system(
//     mut event_start: EventReader<AutoAttackStart>,
//     mut event_launch: EventWriter<AutoAttackLaunch>,
// ) {
//     for event in event_start.read() {
//         event_launch.write(AutoAttackLaunch::new(
//             event.caster,
//             event.target,
//             event.spawn_location,
//             event.source.to_owned(),
//         ));
//     }
// }

// pub fn autoattack_system(
//     mut commands: Commands,
//     attackers: Query<(Entity, &AutoAttack)>,
//     victims: Query<Entity>,

//     mut reader: EventMutator<AutoAttackLaunch>,
// ) {
//     for event in reader.read() {
//         if let Ok((caster, aa)) = attackers.get(event.caster) {
//             let victim = victims.get(event.target).unwrap();
//             let amount = aa.amount.value();
//             let mut missile = commands.spawn((
//                 MissileBundle::new(
//                     RenderBundle {
//                         position: Position::new(
//                             event.spawn_location.x,
//                             event.spawn_location.y,
//                             event.spawn_location.z,
//                         ),
//                         size: Size {
//                             x: 0.25.into(),
//                             y: 0.05.into(),
//                             z: 0.05.into(),
//                         },
//                         ..Default::default()
//                     },
//                     Missile::new(
//                         caster,
//                         MissileTarget::Entity(victim),
//                         vec![Box::new(DamageEffect {
//                             amount,
//                             crit: false,
//                         })],
//                     ),
//                     3.0,
//                 ),
//                 ThisIsAutoAttackMissile,
//             ));
//             event.source = AutoAttackSource::Missile(missile.id());

//             for insert in event.inserts.drain(..) {
//                 insert(&mut missile);
//             }
//         }
//     }
// }

// pub fn missile_system(
//     mut commands: Commands,
//     mut missiles: Query<(Entity, &mut Missile, &Position)>,
//     targets: Query<&Position, Without<Missile>>,
//     mut event: EventWriter<MissileHit>,
// ) {
//     for (entity, mut missile, pos) in missiles.iter_mut() {
//         if let MissileTarget::Entity(target) = missile.target {
//             if let Ok(enemy_pos) = targets.get(target) {
//                 commands.entity(entity).insert(Destination::new(
//                     enemy_pos.x(),
//                     enemy_pos.y(),
//                     enemy_pos.z(),
//                 ));

//                 let dx = pos.x() - enemy_pos.x();
//                 let dy = pos.y() - enemy_pos.y();
//                 let dz = pos.z() - enemy_pos.z();
//                 let dist = (dx * dx + dy * dy + dz * dz).sqrt();

//                 if dist < 0.1 {
//                     let caster = missile.caster;
//                     event.write(MissileHit::new(caster, target, entity));
//                 }
//             }
//         } else if let MissileTarget::Position(target) = missile.target {
//             commands
//                 .entity(entity)
//                 .insert(Destination::new(target.x, target.y, target.z));

//             let dx = pos.x() - target.x;
//             let dy = pos.y() - target.y;
//             let dz = pos.z() - target.z;
//             let dist = (dx * dx + dy * dy + dz * dz).sqrt();

//             if dist < 0.1 {
//                 let caster = missile.caster;

//                 for effect in &mut missile.impact {
//                     effect.play(caster, MissileTarget::Position(target), &mut commands);
//                 }

//                 commands.entity(entity).despawn();
//             }
//         }
//     }
// }

// pub fn missile_hit_system(
//     mut commands: Commands,
//     mut missiles: Query<(Entity, &mut Missile, Option<&ThisIsAutoAttackMissile>)>,

//     mut event_reader: EventReader<MissileHit>,
//     mut event_writer: EventWriter<AutoAttackHit>,
// ) {
//     for event in event_reader.read() {
//         if let Ok((entity, mut missile, is_auto_attack)) = missiles.get_mut(event.source) {
//             if event.is_deleted {
//                 continue;
//             }
//             for effect in &mut missile.impact {
//                 effect.play(
//                     event.caster,
//                     MissileTarget::Entity(event.target),
//                     &mut commands,
//                 );
//             }

//             if is_auto_attack.is_some() {
//                 event_writer.write(AutoAttackHit::new(
//                     event.caster,
//                     event.target,
//                     AutoAttackSource::Missile(event.source),
//                 ));
//             }

//             // commands.entity(entity).despawn();
//             commands.entity(entity).insert(PendingDespawn);
//             // if let Ok(mut ec) = commands.get_entity(entity) {
//             //     ec.insert(PendingDespawn);
//             // }
//         }
//     }
// }

// pub fn despawn_pending_system(mut commands: Commands, query: Query<Entity, With<PendingDespawn>>) {
//     for entity in query {
//         commands.entity(entity).despawn();
//     }
// }

// pub fn pending_damage_system(
//     mut commands: Commands,
//     query: Query<(Entity, &PendingDamage)>,
//     mut writer: EventWriter<Damage>,
// ) {
//     for (entity, pending_damage) in query {
//         writer.write(Damage::new(
//             pending_damage.caster,
//             pending_damage.target,
//             pending_damage.amount,
//             pending_damage.crit,
//         ));

//         commands.entity(entity).despawn();
//     }
// }

// // pub fn ability_system(
// //     mut commands: Commands,
// //     keyboard: Res<Keyboard>,
// //     mut query: Query<(Entity, &mut Abilities), Without<CantUseAbilities>>,
// //     mut ev_start: EventWriter<AbilityCastStarted>,
// //     mut ev_succ: EventWriter<AbilityCastSucceeded>,
// //     mut ev_fail: EventWriter<AbilityCastFailed>,
// // ) {
// //     let now = Instant::now();

// //     for (caster, mut abilities) in query.iter_mut() {
// //         for ability in abilities.list.iter_mut() {
// //             if keyboard.is_key_clicked(ability.key) {
// //                 ev_start.write(AbilityCastStarted {
// //                     caster,
// //                     key: ability.key,
// //                 });

// //                 if now.duration_since(ability.last_cast) < ability.cooldown {
// //                     ev_fail.write(AbilityCastFailed {
// //                         caster,
// //                         key: ability.key,
// //                     });
// //                     continue;
// //                 }

// //                 let mut ok = true;
// //                 for action in ability.actions.iter_mut() {
// //                     if !action.play(caster, &mut world) {
// //                         ok = false;
// //                         break;
// //                     }
// //                 }

// //                 if ok {
// //                     ability.last_cast = now;
// //                     ev_succ.send(AbilityCastSucceeded {
// //                         caster,
// //                         key: ability.key,
// //                     });
// //                 } else {
// //                     ev_fail.send(AbilityCastFailed {
// //                         caster,
// //                         key: ability.key,
// //                     });
// //                 }
// //             }
// //         }
// //     }
// // }

// pub fn lifespan_system(mut commands: Commands, query: Query<(Entity, &Lifespan)>) {
//     let now = Instant::now();

//     for (entity, lifespan) in &query {
//         if now.duration_since(lifespan.spawned) >= lifespan.duration {
//             commands.entity(entity).despawn();
//         }
//     }
// }

// pub fn ability_system(world: &mut World) {
//     let mut to_activate = Vec::new();

//     let events = world.resource::<Events<UseAbilityEvent>>();

//     for ev in events.iter_current_update_events() {
//         if let Some(abilities) = world.get::<Abilities>(ev.entity)
//             && let Some(ability) = abilities.list.get(&ev.key)
//         {
//             to_activate.push((ability.clone(), ev.entity));
//         }
//     }

//     for (ability, entity) in to_activate {
//         ability.activate(world, entity);
//     }

//     world.resource_mut::<Events<UseAbilityEvent>>().clear();
// }

// pub fn input_system(
//     mut writer: EventWriter<UseAbilityEvent>,
//     query: Query<Entity, With<Player>>,
//     keyboard: Res<Keyboard>,
// ) {
//     for key in keyboard.released_keys() {
//         if let Ok(entity) = query.single() {
//             writer.write(UseAbilityEvent { entity, key });
//         }
//     }
// }
