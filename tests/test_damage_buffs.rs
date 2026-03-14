use heroes_of_the_storm_on_rust::prelude::*;

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Component, Clone)]
    struct DamageBuff {
        buff: Reversible<i32>,
    }

    fn apply_damage_buffs(mut events: ResMut<Events<DamageEvent>>, query: Query<&DamageBuff>) {
        let mut modified: Vec<DamageEvent> = Vec::new();

        for event in events.drain() {
            let mut amount = event.amount;

            if let Ok(buff) = query.get(event.target) {
                amount += buff.buff.value().into();
            }

            modified.push(DamageEvent {
                target: event.target,
                amount: amount,
            });
        }

        for ev in modified {
            events.send(ev);
        }
    }

    #[derive(Component)]
    struct Health {
        current: Reversible<i32>,
        _max: Reversible<i32>,
    }
    #[derive(Event, Clone)]
    struct DamageEvent {
        target: Entity,
        amount: Reversible<i32>,
    }

    fn spawn_damage(mut writer: EventWriter<DamageEvent>, query: Query<Entity, With<Health>>) {
        for target in query {
            writer.write(DamageEvent {
                target,
                amount: 25.into(),
            });
        }
    }

    fn apply_damage(
        mut query: Query<(Entity, &mut Health)>,
        mut events: ResMut<Events<DamageEvent>>,
    ) {
        for event in events.drain() {
            if let Ok((_entity, mut health)) = query.get_mut(event.target) {
                let amount = event.amount.value() as i32;
                health.current -= amount.into();
            }
        }
    }

    fn test_dmg(mut _commands: Commands, query: Query<(Entity, &mut Health)>) {
        for (_entity, health) in query {
            println!("health: {}", health.current.value());
        }
    }

    #[test]
    fn test_damage() {
        let mut world = World::new();
        world.insert_resource(Events::<DamageEvent>::default());

        let mut schedule = Schedule::default();
        schedule.add_systems((
            spawn_damage,
            apply_damage_buffs.after(spawn_damage),
            apply_damage.after(apply_damage_buffs),
            test_dmg.after(apply_damage),
        ));

        world.spawn((
            Health {
                current: 100.into(),
                _max: 100.into(),
            },
            DamageBuff { buff: 10.into() },
        ));

        schedule.run(&mut world);
        schedule.run(&mut world);
        schedule.run(&mut world);
        schedule.run(&mut world);

        // assert_eq!(entity_mut.get::<Health>().unwrap().current, 100);
        assert_eq!(true, false);
    }
}
