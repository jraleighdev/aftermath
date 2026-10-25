use bevy::prelude::*;

use crate::player::Player;
/// A projectile: pixels per second it travels along its velocity.
#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
}

/// Bullet speed in pixels per second.
pub const SPEED: f32 = 600.0;

/// Spawns a bullet from every `Player` along its aim direction while the
/// left mouse button is held (autofire covers click and hold).
pub fn shoot(
    mouse: Res<ButtonInput<MouseButton>>,
    players: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    for transform in &players {
        let direction = transform.right().xy();
        commands.spawn((
            Bullet { velocity: direction * SPEED },
            Transform::from_translation(transform.translation),
            Sprite::from_color(Color::srgb(1.0, 0.2, 0.2), Vec2::new(12.0, 4.0)),
        ));
    }
}

/// Moves every bullet along its velocity.
pub fn move_bullets(time: Res<Time>, mut bullets: Query<(&Bullet, &mut Transform)>) {
    for (bullet, mut transform) in &mut bullets {
        transform.translation += bullet.velocity.extend(0.0) * time.delta_secs();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;
    use std::time::Duration;

    /// World with a 1s-old clock, one player at the origin facing `angle`,
    /// and the left mouse button pressed iff `pressed`.
    fn shoot_world(pressed: bool, angle: f32) -> World {
        let mut world = World::default();

        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);

        let mut mouse = ButtonInput::<MouseButton>::default();
        if pressed {
            mouse.press(MouseButton::Left);
        }
        world.insert_resource(mouse);

        world.spawn((
            Player,
            Transform::from_rotation(Quat::from_rotation_z(angle)),
        ));
        world
    }

    fn run(world: &mut World, mark_from_schedule: fn(&mut Schedule)) {
        let mut schedule = Schedule::default();
        mark_from_schedule(&mut schedule);
        schedule.run(world);
    }

    fn add_shoot(schedule: &mut Schedule) {
        schedule.add_systems(shoot);
    }

    fn add_move(schedule: &mut Schedule) {
        schedule.add_systems(move_bullets);
    }

    fn bullet_count(world: &mut World) -> usize {
        world.query::<&Bullet>().iter(&world).count()
    }

    #[test]
    fn shoots_toward_aim_when_pressed() {
        // Player faces +X (zero rotation, click or hold): bullet flies right.
        let mut world = shoot_world(true, 0.0);
        run(&mut world, add_shoot);
        assert_eq!(bullet_count(&mut world), 1);

        let query = world.query::<(&Bullet, &Transform)>().single(&world).unwrap();
        assert!(query.0.velocity.abs_diff_eq(Vec2::new(SPEED, 0.0), 0.01));
        assert!(query.1.translation.xy().abs_diff_eq(Vec2::ZERO, 0.01));
    }

    #[test]
    fn shoots_toward_aim_when_facing_up() {
        // Facing +Y: bullet flies up. Held input counts too (click/hold).
        let mut world = shoot_world(true, FRAC_PI_2);
        run(&mut world, add_shoot);
        let query = world.query::<(&Bullet, &Transform)>().single(&world).unwrap();
        assert!(query.0.velocity.abs_diff_eq(Vec2::new(0.0, SPEED), 0.01));
    }

    #[test]
    fn shoots_nothing_without_input() {
        let mut world = shoot_world(false, 0.0);
        run(&mut world, add_shoot);
        assert_eq!(bullet_count(&mut world), 0);
    }

    #[test]
    fn bullets_move_along_their_direction() {
        let mut world = World::default();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);
        world.spawn((Bullet { velocity: Vec2::new(SPEED, 0.0) }, Transform::default()));

        run(&mut world, add_move);
        let translation = world.query::<&Transform>().single(&world).unwrap().translation;
        assert!(translation.xy().abs_diff_eq(Vec2::new(SPEED, 0.0), 0.01));

        // Turning left (π) sends it back the other way.
        let mut world = World::default();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);
        world.spawn((Bullet { velocity: Vec2::new(-SPEED, 0.0) }, Transform::default()));
        run(&mut world, add_move);
        let translation = world.query::<&Transform>().single(&world).unwrap().translation;
        assert!(translation.xy().abs_diff_eq(Vec2::new(-SPEED, 0.0), 0.01));
    }
}
