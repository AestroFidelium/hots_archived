// use crate::prelude::*;

// #[derive(Component)]
// pub struct TimeRewind {
//     pub iter_x: Vec<f32>,
//     pub iter_y: Vec<f32>,
//     pub iter_z: Vec<f32>,
//     pub iter_health: Vec<f32>,
//     pub rotate_yaw: Vec<f32>,
//     pub step: usize,
// }

// implement_new_tag!(
//     PlayerGhost,
//     TracerQ,
//     TracerW,
//     TracerR,
//     TracerHealAA,
//     Ricochet,
//     ThisIsAlreadyRicocheted,
//     IsDeflecting
// );

// implement_tag_with_fields!(Vampirism(strength:f32), IncreaseDamage(strength:f32) );

// #[derive(Component)]
// pub struct TracerQAnimation {
//     pub from: Vector3<f32>,
//     pub to: Vector3<f32>,
//     pub step: usize,
//     pub total_steps: usize,
// }

// pub struct TracerREffect {}

// impl Effect for TracerREffect {
//     fn play(&mut self, _caster: Entity, _target: MissileTarget, _commands: &mut Commands) {
//         // commands.spawn(bundle)
//     }
// }
