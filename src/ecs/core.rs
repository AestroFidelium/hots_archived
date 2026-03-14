use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

use cgmath::{Deg, InnerSpace, Matrix4, Point3, SquareMatrix, Vector3, Vector4, perspective, vec3};
use derive_more::Debug;
use uuid::Uuid;
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{
    ecs::{Destination, Position, Reversible, Rotation},
    support::{ModelData, SCREEN_HEIGHT, SCREEN_WIDTH},
};

use bevy_ecs::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum CoreSets {
    Initializing,
    Updating,
    Buffs,
    Apply,
    Rendering,
    Pending,
    Clearing,
}

#[macro_export]
macro_rules! implement_new_tag {
    ($($name:ident),*) => {
        $(
            #[derive(Component, Debug, Clone, Default)]
            pub struct $name;
        )*
    };
}

#[macro_export]
macro_rules! implement_tag_with_fields {
    ($( $name:ident $( ( $($fields:ident : $ty:ty),* $(,)? ) )? ),* $(,)? ) => {
        $(
            #[derive(Component, Debug, Clone, Default)]
            pub struct $name {
                $(
                    $(pub $fields: $ty,)*
                )?
            }
        )*
    };
}

implement_new_tag!(
    Renderable,
    Player,
    IsAlive,
    IsDead,
    IsMoving,
    IsRotating,
    CanMoving,
    IsAutoAttacking,
    IsStanding,
    IsTargetedByAutoAttack,
    IsSilenced,
    CantUseAbilities,
    ThisIsAutoAttackMissile
);

#[derive(Bundle, Debug, Clone, Default)]
pub struct UnitTagsBundle {
    is_alive: IsAlive,
    can_moving: CanMoving,
    is_standing: IsStanding,
}

#[derive(Component, Debug, Clone)]
pub struct Team {
    pub player: i32,
}

impl Default for Team {
    fn default() -> Self {
        Self { player: 1 }
    }
}

#[derive(Bundle, Debug, Clone, Default)]
pub struct RenderBundle {
    pub position: Position,
    pub rotation: Rotation,
    pub size: Size,
    pub color: Color,
    pub rendable: Renderable,
}

impl RenderBundle {
    pub fn new(position: [f32; 3], rotation: [f32; 3], size: [f32; 3], color: [f32; 4]) -> Self {
        Self {
            position: Position::new(position[0], position[1], position[2]),

            rotation: Rotation::new(rotation[0], rotation[1], rotation[2]),
            size: Size::new(size[0], size[1], size[2]),
            color: Color::new(color[0], color[1], color[2], color[3]),
            rendable: Renderable,
        }
    }
}

#[derive(Bundle, Debug, Clone, Default)]
pub struct UnitBundle {
    pub render: RenderBundle,
    pub movement_speed: MovementSpeed,
    pub health: Health,
    pub team: Team,
    pub autoattack: AutoAttack,

    pub tags: UnitTagsBundle,
}

#[derive(Resource, Debug, Clone)]
pub struct RenderModels(pub Vec<ModelData>);

#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
    pub color: Color,
    pub scale: f32,
    pub position: Vector3<f32>,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            text: Default::default(),
            color: Default::default(),
            scale: Default::default(),
            position: Vector3::new(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ResourceWorldText {
    pub texts: Vec<Text>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ResourceScreenText {
    pub texts: Vec<Text>,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub alpha: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, alpha: f32) -> Self {
        Self { r, g, b, alpha }
    }

    pub fn get_color(&self) -> [f32; 4] {
        [self.r, self.g, self.b, self.alpha]
    }

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        // убираем '#' если есть
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 {
            return Err(format!("Invalid hex color length: {}", hex.len()));
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;

        Ok(Self {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            alpha: 1.0,
        })
    }
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            alpha: 1.0,
        }
    }
}

#[derive(Default)]
pub struct Mouse {
    pub position: [f64; 2],
    pub pressed: HashSet<MouseButton>,
    pub released: HashSet<MouseButton>,
    pub held: HashSet<MouseButton>,
}

impl Mouse {
    pub fn new() -> Self {
        Self {
            position: [0.0, 0.0],
            pressed: HashSet::new(),
            released: HashSet::new(),
            held: HashSet::new(),
        }
    }

    pub fn is_button_clicked(&self, button: MouseButton) -> bool {
        self.pressed.contains(&button)
    }

    pub fn is_button_released(&self, button: MouseButton) -> bool {
        self.released.contains(&button)
    }

    pub fn is_button_hold(&self, button: MouseButton) -> bool {
        self.held.contains(&button)
    }

    pub fn clear_frame(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }
}

#[derive(Default)]
pub struct Keyboard {
    pub pressed: HashSet<KeyCode>,
    pub released: HashSet<KeyCode>,
    pub held: HashSet<KeyCode>,
}

impl Keyboard {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            released: HashSet::new(),
            held: HashSet::new(),
        }
    }

    pub fn clear_frame(&mut self) {
        self.pressed.clear();
        self.released.clear();
    }

    pub fn is_key_clicked(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn pressed_keys(&self) -> Vec<KeyCode> {
        self.pressed.iter().copied().collect()
    }

    pub fn is_key_released(&self, key: KeyCode) -> bool {
        self.released.contains(&key)
    }

    pub fn released_keys(&self) -> Vec<KeyCode> {
        self.released.iter().copied().collect()
    }

    pub fn is_key_hold(&self, key: KeyCode) -> bool {
        self.held.contains(&key)
    }

    pub fn held_keys(&self) -> Vec<KeyCode> {
        self.held.iter().copied().collect()
    }
}

#[derive(Component, Debug, Clone)]
pub struct MovementSpeed {
    current: Reversible<f32>,
}

impl MovementSpeed {
    pub fn new(current: f32) -> Self {
        Self {
            current: current.into(),
        }
    }
    pub fn current(&self) -> f32 {
        self.current.value()
    }
    pub fn current_ref(&self) -> &Reversible<f32> {
        &self.current
    }
}

impl Default for MovementSpeed {
    fn default() -> Self {
        Self {
            current: (3.0).into(),
        }
    }
}

#[derive(Clone)]
pub struct Camera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,

    pub move_speed: f32,
    pub zoom_speed: f32,
    pub min_height: f32,
    pub max_height: f32,

    pub view_width: f32,  // Базовая ширина вида на уровне земли
    pub view_height: f32, // Базовая высота вида
    pub zoom: f32,        // Масштаб (1.0 - нормальный, >1 - приближение)
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Point3::new(0.0, 15.0, 8.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
            move_speed: 0.3,
            zoom_speed: 5.0,
            min_height: 10.0,
            max_height: 100.0,

            view_width: 40.0, // Базовый размер вида
            view_height: 30.0,
            zoom: 1.0,
        }
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        Matrix4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        let aspect = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
        perspective(Deg(45.0), aspect, 0.1, 1000.0)
    }

    pub fn update(&mut self, cursor_x: f64, cursor_y: f64) {
        let edge = 10.0;

        if cursor_x <= edge {
            self.position.x -= self.move_speed;
            self.target.x -= self.move_speed;
        }
        if cursor_x >= (SCREEN_WIDTH as f64 - edge) {
            self.position.x += self.move_speed;
            self.target.x += self.move_speed;
        }
        if cursor_y <= edge {
            self.position.z -= self.move_speed;
            self.target.z -= self.move_speed;
        }
        if cursor_y >= (SCREEN_HEIGHT as f64 - edge) {
            self.position.z += self.move_speed;
            self.target.z += self.move_speed;
        }
    }

    pub fn zoom(&mut self, delta: f32) {
        self.position.y -= delta * self.zoom_speed;

        if self.position.y < self.min_height {
            self.position.y = self.min_height;
        }
        if self.position.y > self.max_height {
            self.position.y = self.max_height;
        }
    }

    pub fn cursor_to_world(&self, cursor_x: f64, cursor_y: f64) -> Option<Point3<f32>> {
        let x = (2.0 * cursor_x as f32) / SCREEN_WIDTH as f32 - 1.0;
        let y = 1.0 - (2.0 * cursor_y as f32) / SCREEN_HEIGHT as f32;
        let ndc = Vector3::new(x, y, 1.0);

        let view = self.view_matrix();
        let proj = self.projection_matrix();
        let inv_vp = (proj * view).invert().unwrap();

        let clip = Vector4::new(ndc.x, ndc.y, -1.0, 1.0);
        let mut world = inv_vp * clip;
        world /= world.w;

        let world_pos = Point3::new(world.x, world.y, world.z);
        let dir = (world_pos - self.position).normalize();

        if dir.y.abs() < 1e-6 {
            return None;
        }
        let t = -self.position.y / dir.y;
        if t < 0.0 {
            return None;
        }

        Some(self.position + dir * t)
    }
}

#[derive(Component, Debug, Clone)]
pub struct Size {
    pub x: Reversible<f32>,
    pub y: Reversible<f32>,
    pub z: Reversible<f32>,
}

impl Size {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    pub fn x(&self) -> f32 {
        self.x.value()
    }
    pub fn x_ref(&self) -> &Reversible<f32> {
        &self.x
    }

    pub fn x_ref_mut(&mut self) -> &mut Reversible<f32> {
        &mut self.x
    }

    pub fn y_ref_mut(&mut self) -> &mut Reversible<f32> {
        &mut self.y
    }

    pub fn z_ref_mut(&mut self) -> &mut Reversible<f32> {
        &mut self.z
    }

    pub fn y(&self) -> f32 {
        self.y.value()
    }
    pub fn y_ref(&self) -> &Reversible<f32> {
        &self.y
    }

    pub fn z(&self) -> f32 {
        self.z.value()
    }
    pub fn z_ref(&self) -> &Reversible<f32> {
        &self.z
    }
}

impl Default for Size {
    fn default() -> Self {
        Self {
            x: (1.0).into(),
            y: (1.0).into(),
            z: (1.0).into(),
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct Health {
    current: Reversible<f32>,
    max: Reversible<f32>,

    #[allow(dead_code)]
    regeneration: Reversible<f32>,
}

impl Health {
    pub fn new(health: f32) -> Self {
        Self {
            current: health.into(),
            max: health.into(),
            regeneration: (0.02035 / 62.5 * health).into(), // 2% health in second (same as hots)
        }
    }

    pub fn current(&self) -> f32 {
        self.current.value()
    }

    pub fn max(&self) -> f32 {
        self.max.value()
    }

    pub fn current_ref(&self) -> &Reversible<f32> {
        &self.current
    }

    pub fn current_mut(&mut self) -> &mut Reversible<f32> {
        &mut self.current
    }
}

impl Default for Health {
    fn default() -> Self {
        Self {
            current: (1000.0).into(),
            max: (1000.0).into(),
            regeneration: (0.02035 / 62.5 * 1000.0).into(),
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct Heal {
    pub caster: Entity,
    pub target: Entity,
    pub basic_heal: f32,
    pub bonus: f32,
}

impl Heal {
    pub fn new(caster: Entity, target: Entity, basic_heal: f32) -> Self {
        Self {
            caster,
            target,
            basic_heal,
            bonus: 0.0,
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct Damage {
    pub attacker: Entity,
    pub target: Entity,
    pub basic_damage: f32,
    pub bonus: f32,
    pub is_crit: bool,
}

impl Damage {
    pub fn new(attacker: Entity, target: Entity, basic_damage: f32, is_crit: bool) -> Self {
        Self {
            attacker,
            target,
            basic_damage,
            bonus: 0.0,
            is_crit,
        }
    }
}

#[derive(Bundle, Debug, Clone, Default)]

pub struct AutoAttackBundle {}

#[derive(Debug, Clone, Default)]
pub enum DamageType {
    #[default]
    Magic,
    Physical,
    Percent,
}
#[derive(Debug, Clone, Default)]
pub enum DamagerType {
    Missile,
    Magic,
    #[default]
    Instant,
}
#[derive(Debug, Clone, Default)]
pub enum AutoAttackRange {
    #[default]
    Melee,
    Range(i32),
}

#[derive(Component)]
pub struct AttackTimer {
    pub last_attack: Instant,
}

#[derive(Component, Debug, Clone, Default)]
pub struct AutoAttack {
    pub amount: Reversible<f32>,
    pub speed: Reversible<f32>,
    pub range: AutoAttackRange,
    pub chance_crit: f32,
    pub chance_miss: f32,
    pub penetration: f32,
    pub damage_type: DamagerType,
}

#[derive(Bundle)]
pub struct MissileBundle {
    pub render: RenderBundle,
    pub missile: Missile,

    pub movement_speed: MovementSpeed,
    pub destination: Destination,

    // tags
    pub can_moving: CanMoving,
    pub is_alive: IsAlive,
}

impl MissileBundle {
    pub fn new(render: RenderBundle, missile: Missile, speed: f32) -> Self {
        Self {
            render,
            missile,
            can_moving: CanMoving,
            movement_speed: MovementSpeed {
                current: speed.into(),
            },
            destination: Destination::default(),
            is_alive: IsAlive,
        }
    }
}

pub trait Effect: Send + Sync {
    fn play(&mut self, caster: Entity, target: MissileTarget, commands: &mut Commands);
}

pub struct DamageEffect {
    pub amount: f32,
    pub crit: bool,
}

#[derive(Component)]
pub struct PendingDamage {
    pub caster: Entity,
    pub target: Entity,
    pub amount: f32,
    pub crit: bool,
}

impl Effect for DamageEffect {
    fn play(&mut self, caster: Entity, target: MissileTarget, commands: &mut Commands) {
        if let MissileTarget::Entity(target) = target {
            commands.spawn(PendingDamage {
                amount: self.amount,
                target,
                caster,
                crit: false,
            });
        }
    }
}

pub enum MissileTarget {
    Entity(Entity),
    Position(Vector3<f32>),
}

#[derive(Component)]
pub struct Missile {
    pub caster: Entity,
    pub target: MissileTarget,
    // pub speed: f32,
    // pub impact: Box<dyn Fn(Entity, Entity) + Send + Sync>,
    pub impact: Vec<Box<dyn Effect>>,
    pub uuid: Uuid,
}

impl Missile {
    pub fn new(caster: Entity, target: MissileTarget, impact: Vec<Box<dyn Effect>>) -> Self {
        Self {
            caster,
            target,
            impact,
            uuid: Uuid::new_v4(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AutoAttackSource {
    Missile(Entity),
    JustCreated,
    Nothing,
}

#[derive(Debug, Clone)]
pub enum AttackType {
    Original,
    Changed,
    Cloned,
}

#[derive(Event, Debug, Clone)]
pub struct AbilityCastStarted {
    pub caster: Entity,
    pub key: KeyCode,
}

#[derive(Event, Debug, Clone)]
pub struct AbilityCastSucceeded {
    pub caster: Entity,
    pub key: KeyCode,
}

#[derive(Event, Debug, Clone)]
pub struct AbilityCastFailed {
    pub caster: Entity,
    pub key: KeyCode,
}

pub fn find_closest_target<'a, F>(
    attacker: Entity,
    attacker_pos: &Position,
    attacker_team: &Team,
    targets: impl Iterator<Item = (Entity, &'a Position, &'a Team)>,
    attack_range: f32,
    mut filter: F,
) -> Option<Entity>
where
    F: FnMut(Entity, &Position, &Team) -> bool,
{
    let mut target: Option<(Entity, f32)> = None;

    for (enemy, enemy_pos, enemy_team) in targets {
        // Базовые исключения
        if enemy == attacker || attacker_team.player == enemy_team.player {
            continue;
        }

        // Кастомный фильтр
        if !filter(enemy, enemy_pos, enemy_team) {
            continue;
        }

        let dist = attacker_pos.distance_calculate(enemy_pos);

        if dist <= attack_range {
            if let Some((_, best_dist)) = target {
                if dist < best_dist {
                    target = Some((enemy, dist));
                }
            } else {
                target = Some((enemy, dist));
            }
        }
    }

    target.map(|(entity, _)| entity)
}

pub fn find_closest_targets<'a, F>(
    attacker: Entity,
    attacker_pos: &Position,
    attacker_team: &Team,
    targets: impl Iterator<Item = (Entity, &'a Position, &'a Team)>,
    attack_range: f32,
    mut filter: F,
    limit: usize,
) -> Vec<Entity>
where
    F: FnMut(Entity, &Position, &Team) -> bool,
{
    let mut candidates: Vec<(Entity, f32)> = targets
        .filter(|(enemy, enemy_pos, enemy_team)| {
            // базовые исключения
            if *enemy == attacker || attacker_team.player == enemy_team.player {
                return false;
            }
            // кастомный фильтр
            filter(*enemy, enemy_pos, enemy_team)
        })
        .filter_map(|(enemy, enemy_pos, _)| {
            let dist = attacker_pos.distance_calculate(enemy_pos);
            if dist <= attack_range {
                Some((enemy, dist))
            } else {
                None
            }
        })
        .collect();

    // сортируем по расстоянию
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // возвращаем только Entity ближайших целей
    candidates
        .into_iter()
        .map(|(entity, _)| entity)
        .take(limit)
        .collect()
}

pub fn find_targets_in_range<'a, F>(
    attacker: Entity,
    attacker_pos: &Position,
    attacker_team: &Team,
    targets: impl Iterator<Item = (Entity, &'a Position, &'a Team)>,
    attack_range: f32,
    mut filter: F,
) -> Vec<Entity>
where
    F: FnMut(Entity, &Position, &Team) -> bool,
{
    let mut candidates: Vec<(Entity, f32)> = targets
        .filter(|(enemy, enemy_pos, enemy_team)| {
            // базовые исключения
            if *enemy == attacker || attacker_team.player == enemy_team.player {
                return false;
            }
            // кастомный фильтр
            filter(*enemy, enemy_pos, enemy_team)
        })
        .filter_map(|(enemy, enemy_pos, _)| {
            let dist = attacker_pos.distance_calculate(enemy_pos);
            if dist <= attack_range {
                Some((enemy, dist))
            } else {
                None
            }
        })
        .collect();

    // сортируем по расстоянию
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    // возвращаем только Entity
    candidates.into_iter().map(|(entity, _)| entity).collect()
}

pub fn find_targets_from_position<'a, F>(
    origin_pos: &Position,
    origin_team: &Team,
    targets: impl Iterator<Item = (Entity, &'a Position, &'a Team)>,
    range: f32,
    mut filter: F,
    limit: usize,
) -> Vec<Entity>
where
    F: FnMut(Entity, &Position, &Team) -> bool,
{
    let mut candidates: Vec<(Entity, f32)> = targets
        .filter(|(enemy, enemy_pos, enemy_team)| {
            // не атакуем союзников
            if origin_team.player == enemy_team.player {
                return false;
            }
            // кастомный фильтр
            filter(*enemy, enemy_pos, enemy_team)
        })
        .filter_map(|(enemy, enemy_pos, _)| {
            let dist = origin_pos.distance_calculate(enemy_pos);
            if dist <= range {
                Some((enemy, dist))
            } else {
                None
            }
        })
        .collect();

    // сортируем по расстоянию
    candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    candidates
        .into_iter()
        .map(|(entity, _)| entity)
        .take(limit)
        .collect()
}

#[derive(Event, Debug, Clone)]
pub struct AutoAttackStart {
    pub caster: Entity,
    pub target: Entity,
    pub spawn_location: Vector3<f32>,
    pub source: AutoAttackSource,
    pub aa_type: AttackType,
}

impl AutoAttackStart {
    pub fn new(
        caster: Entity,
        target: Entity,
        spawn_location: Vector3<f32>,
        source: AutoAttackSource,
        aa_type: AttackType,
    ) -> Self {
        Self {
            caster,
            target,
            spawn_location,
            source,
            aa_type,
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct AutoAttackHit {
    pub caster: Entity,
    pub target: Entity,
    pub source: AutoAttackSource,
}

impl AutoAttackHit {
    pub fn new(caster: Entity, target: Entity, source: AutoAttackSource) -> Self {
        Self {
            caster,
            target,
            source,
        }
    }
}

type InsertFunc = Box<dyn FnOnce(&mut EntityCommands) + Send + Sync>;

#[derive(Event)]
pub struct AutoAttackLaunch {
    pub caster: Entity,
    pub target: Entity,
    pub spawn_location: Vector3<f32>,
    pub source: AutoAttackSource,
    pub inserts: Vec<InsertFunc>,
}

impl AutoAttackLaunch {
    pub fn new(
        caster: Entity,
        target: Entity,
        spawn_location: Vector3<f32>,
        source: AutoAttackSource,
    ) -> Self {
        Self {
            caster,
            target,
            spawn_location,
            source,
            inserts: vec![],
        }
    }

    pub fn with_insert<T: Bundle + 'static>(mut self, bundle: T) -> Self {
        self.inserts.push(Box::new(move |ec: &mut EntityCommands| {
            ec.insert(bundle);
        }));
        self
    }
}

#[derive(Event, Debug, Clone)]
pub struct AutoAttackEnd {
    pub caster: Entity,
    pub target: Entity,
    pub spawn_location: Vector3<f32>,
    pub source: AutoAttackSource,
}

impl AutoAttackEnd {
    pub fn new(
        caster: Entity,
        target: Entity,
        spawn_location: Vector3<f32>,
        source: AutoAttackSource,
    ) -> Self {
        Self {
            caster,
            target,
            spawn_location,
            source,
        }
    }
}

#[derive(Event, Debug, Clone)]
pub struct MissileHit {
    pub caster: Entity,
    pub target: Entity,
    pub source: Entity,
    pub is_deleted: bool,
}

impl MissileHit {
    pub fn new(caster: Entity, target: Entity, source: Entity) -> Self {
        Self {
            caster,
            target,
            source,
            is_deleted: false,
        }
    }
}

#[derive(Component)]
pub struct PendingDespawn;

#[derive(Component)]
pub struct Lifespan {
    pub spawned: Instant,
    pub duration: Duration,
}

impl Lifespan {
    pub fn new(duration: Duration) -> Self {
        Self {
            spawned: Instant::now(),
            duration,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct WorldText {
    pub text: String,
    pub scale: f32,
}

#[derive(Component, Debug, Clone)]
pub struct ScreenText {
    pub text: String,
    pub color: Color,
    pub scale: f32,
    pub position: Vector3<f32>,
}

// #[derive(Debug, Clone)]
// pub struct HealthChangingText {
//     pub text: String,
//     pub color: Color,
//     pub scale: f32,
//     pub position: Vector3<f32>,
// }

#[derive(Component)]
pub struct HealthChangeText {
    pub owner: Entity,
    pub value: f32,
}

pub trait Ability: Send + Sync {
    fn activate(&self, world: &mut World, entity: Entity);
    fn box_clone(&self) -> Box<dyn Ability>;
}

impl Clone for Box<dyn Ability> {
    fn clone(&self) -> Box<dyn Ability> {
        self.box_clone()
    }
}

#[derive(Component)]
pub struct Abilities {
    pub list: HashMap<KeyCode, Box<dyn Ability>>,
}

#[derive(Event, Clone)]
pub struct UseAbilityEvent {
    pub entity: Entity, // кто использует
    pub key: KeyCode,   // индекс способности в Vec
}
