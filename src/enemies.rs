use bevy::prelude::*;

use crate::player::{Player, RADIUS as PLAYER_RADIUS};
/// Marker for a hostile enemy.
#[derive(Component)]
pub struct Enemy;

/// Enemy movement speed in pixels per second.
pub const ENEMY_SPEED: f32 = 120.0;

/// Collision radius: half the enemy sprite's size.
pub const RADIUS: f32 = 30.0;

/// Seconds between enemy spawns.
pub const SPAWN_INTERVAL: f32 = 2.0;

/// How far from the player new enemies appear.
pub const SPAWN_DISTANCE: f32 = 500.0;

/// Contact shove speed in pixels per second, applied per frame of contact.
pub const CONTACT_SPEED: f32 = 400.0;

/// Moves every `Enemy` toward the player at `ENEMY_SPEED`.
pub fn chase(
    time: Res<Time>,
    player: Single<&Transform, (With<Player>, Without<Enemy>)>,
    mut enemies: Query<&mut Transform, With<Enemy>>,
) {
    let player_pos = player.translation;
    for mut transform in &mut enemies {
        let direction = (player_pos - transform.translation).normalize_or_zero();
        transform.translation += direction * ENEMY_SPEED * time.delta_secs();
    }
}

/// Shoves the player away from any enemy in contact with them, at
/// `CONTACT_SPEED` per frame of contact so overlap keeps resolving.
pub fn shove_player(
    time: Res<Time>,
    mut player: Single<&mut Transform, (With<Player>, Without<Enemy>)>,
    enemies: Query<&Transform, With<Enemy>>,
) {
    for enemy_pos in &enemies {
        let offset = player.translation.xy() - enemy_pos.translation.xy();
        if offset.length() < PLAYER_RADIUS + RADIUS {
            let direction = offset.normalize_or_zero();
            player.translation += direction.extend(0.0) * CONTACT_SPEED * time.delta_secs();
        }
    }
}

/// Spawns one enemy every `SPAWN_INTERVAL` at `SPAWN_DISTANCE` from the
/// player, ring-style so spawn points vary without an RNG dependency.
pub fn spawn_enemies(
    time: Res<Time>,
    player: Single<&Transform, With<Player>>,
    mut commands: Commands,
    mut accumulator: Local<f32>,
    mut spawn_index: Local<u32>,
) {
    *accumulator += time.delta_secs();
    if *accumulator < SPAWN_INTERVAL {
        return;
    }
    *accumulator -= SPAWN_INTERVAL;
    // Golden-angle ring keeps consecutive spawns spread apart.
    let angle = (*spawn_index % 64) as f32 * 2.399963;
    *spawn_index += 1;
    let offset = Vec2::from_angle(angle) * SPAWN_DISTANCE;
    commands.spawn((
        Enemy,
        Transform::from_translation((player.translation.xy() + offset).extend(0.0)),
        Sprite::from_color(Color::srgb(0.9, 0.3, 0.2), Vec2::splat(60.0)),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// World with a 1s-old clock, one player at the origin, and one enemy
    /// at `pos`.
    fn chase_world(pos: Vec2) -> (World, Entity) {
        let mut world = World::default();

        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);

        world.spawn((Player, Transform::default()));
        let enemy = world.spawn((Enemy, Transform::from_translation(pos.extend(0.0)))).id();
        (world, enemy)
    }

    fn run(world: &mut World, mark_from_schedule: fn(&mut Schedule)) {
        let mut schedule = Schedule::default();
        mark_from_schedule(&mut schedule);
        schedule.run(world);
    }

    fn add_chase(schedule: &mut Schedule) {
        schedule.add_systems(chase);
    }

    #[test]
    fn enemy_steps_enemy_speed_toward_player() {
        let (mut world, enemy) = chase_world(Vec2::new(400.0, 300.0));
        run(&mut world, add_chase);
        let direction = -Vec3::new(400.0, 300.0, 0.0).normalize();
        let expected = Vec3::new(400.0, 300.0, 0.0) + direction * ENEMY_SPEED;
        let translation = world.get::<Transform>(enemy).unwrap().translation;
        assert!(translation.abs_diff_eq(expected, 0.01));
    }

    #[test]
    fn enemies_stay_put_without_a_player() {
        let mut world = World::default();
        world.insert_resource(Time::<()>::default());
        let enemy = world.spawn((Enemy, Transform::from_translation(Vec3::X))).id();

        run(&mut world, add_chase);
        assert_eq!(world.get::<Transform>(enemy).unwrap().translation, Vec3::X);
    }

    #[test]
    fn contact_shoves_the_player_away() {
        let mut world = World::default();
        world.insert_resource(Time::<()>::default());
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_secs(1));
        world.spawn((Player, Transform::default()));
        // 70px apart, within RADIUS_PLAYER + RADIUS_ENEMY = 80.
        world.spawn((Enemy, Transform::from_translation(Vec3::X * 70.0)));

        let mut schedule = Schedule::default();
        schedule.add_systems(shove_player);
        schedule.run(&mut world);

        let translation = world
            .query_filtered::<&Transform, With<Player>>()
            .single(&world)
            .unwrap()
            .translation;
        assert!(translation.abs_diff_eq(Vec3::NEG_X * CONTACT_SPEED, 0.01));
    }

    #[test]
    fn no_contact_leaves_the_player_alone() {
        let mut world = World::default();
        world.insert_resource(Time::<()>::default());
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_secs(1));
        world.spawn((Player, Transform::default()));
        // 200px apart, well outside the 80px contact sum.
        world.spawn((Enemy, Transform::from_translation(Vec3::X * 200.0)));

        let mut schedule = Schedule::default();
        schedule.add_systems(shove_player);
        schedule.run(&mut world);

        let translation = world
            .query_filtered::<&Transform, With<Player>>()
            .single(&world)
            .unwrap()
            .translation;
        assert_eq!(translation, Vec3::ZERO);
    }

    #[test]
    fn shove_direction_respects_geometry() {
        let mut world = World::default();
        world.insert_resource(Time::<()>::default());
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_secs(1));
        world.spawn((Player, Transform::default()));
        // Enemy to the right -> player must move left, nowhere else.
        world.spawn((Enemy, Transform::from_translation(Vec3::X * 70.0)));

        let mut schedule = Schedule::default();
        schedule.add_systems(shove_player);
        schedule.run(&mut world);

        let translation = world
            .query_filtered::<&Transform, With<Player>>()
            .single(&world)
            .unwrap()
            .translation;
        assert!(translation.x < 0.0);
        assert_eq!(translation.y, 0.0);
    }

    /// A spawn accumulator ticks down and spawns one enemy per interval.
    #[test]
    fn spawns_an_enemy_after_the_interval() {
        let mut world = World::default();
        world.insert_resource(Time::<()>::default());
        world.spawn((Player, Transform::default()));
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_enemies);

        // 1s into the 2s interval -> nothing yet.
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_secs(1));
        schedule.run(&mut world);
        assert_eq!(world.query::<&Enemy>().iter(&world).count(), 0);

        // 1.5s more -> crossed the interval -> one enemy.
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(1500));
        schedule.run(&mut world);
        assert_eq!(world.query::<&Enemy>().iter(&world).count(), 1);
    }

    #[test]
    fn spawned_enemy_starts_offscreen_around_the_player() {
        let mut world = World::default();
        world.insert_resource(Time::<()>::default());
        world.spawn((Player, Transform::default()));
        let mut schedule = Schedule::default();
        schedule.add_systems(spawn_enemies);
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis((SPAWN_INTERVAL * 1000.0) as u64));
        schedule.run(&mut world);
        let pos = world
            .query::<(Entity, &Transform)>()
            .iter(&world)
            .map(|(_, t)| t.translation.xy())
            .find(|p| !p.abs_diff_eq(Vec2::ZERO, 0.01))
            .unwrap();
        assert!(pos.length() >= SPAWN_DISTANCE);
        assert!(pos.length() <= 800.0); // near the 800px screen half-diagonal
    }
}
