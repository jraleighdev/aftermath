use bevy::prelude::*;

/// Marker for the player entity.
#[derive(Component)]
pub struct Player;

/// Movement speed in pixels per second.
pub const SPEED: f32 = 300.0;

/// Moves every `Player` with WASD or the arrow keys, normalized so
/// diagonals aren't faster.
pub fn movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut players: Query<&mut Transform, With<Player>>,
) {
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        dir.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        dir.y -= 1.0;
    }
    let step = dir.normalize_or_zero() * SPEED * time.delta_secs();
    for mut transform in &mut players {
        transform.translation += step.extend(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// World with a 1s-old clock, the given keys held, and one player at the origin.
    fn test_world(pressed_keys: &[KeyCode]) -> (World, Entity) {
        let mut world = World::default();

        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        world.insert_resource(time);

        let mut keys = ButtonInput::<KeyCode>::default();
        for key in pressed_keys {
            keys.press(*key);
        }
        world.insert_resource(keys);

        let player = world.spawn((Player, Transform::default())).id();
        (world, player)
    }

    fn run_movement(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(movement);
        schedule.run(world);
    }

    #[test]
    fn moves_right_one_speed_per_second_when_d_pressed() {
        let (mut world, player) = test_world(&[KeyCode::KeyD]);
        run_movement(&mut world);
        assert_eq!(world.get::<Transform>(player).unwrap().translation.x, SPEED);
    }

    #[test]
    fn stays_put_with_no_input() {
        let (mut world, player) = test_world(&[]);
        run_movement(&mut world);
        assert_eq!(
            world.get::<Transform>(player).unwrap().translation,
            Vec3::ZERO
        );
    }
}
