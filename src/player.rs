use bevy::prelude::*;

/// Marker for the player entity.
#[derive(Component)]
pub struct Player;

/// Movement speed in pixels per second.
pub const SPEED: f32 = 300.0;

/// Collision radius: half the player sprite's size.
pub const RADIUS: f32 = 50.0;

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

/// Rotates every `Player` to face the mouse cursor. The square sprite
/// faces right by default, so no extra 90° offset is needed.
pub fn aim(
    mut players: Query<&mut Transform, With<Player>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (camera, camera_transform) = *camera_query;
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };
    for mut transform in &mut players {
        transform.rotation =
            Quat::from_rotation_z((cursor_world - transform.translation.xy()).to_angle());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::RenderTargetInfo;
    use bevy::window::WindowResolution;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};
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

    /// World with an 800x600 window, an identity camera, a cursor at the
    /// given logical position, and one player at the origin.
    fn aim_world(cursor_pos: Option<Vec2>) -> (World, Entity) {
        let mut world = World::default();
        world.init_resource::<Time>();

        let mut window = Window {
            resolution: WindowResolution::new(800, 600),
            ..default()
        };
        window.set_cursor_position(cursor_pos);
        world.spawn(window);

        // Fake what `camera_update` normally does per frame: register the
        // render target so `viewport_to_world_2d` resolves headlessly.
        let camera = world
            .spawn((Camera2d, GlobalTransform::default()))
            .id();
        world.get_mut::<Camera>(camera).unwrap().computed.target_info =
            Some(RenderTargetInfo {
                physical_size: UVec2::new(800, 600),
                scale_factor: 1.0,
            });
        let player = world.spawn((Player, Transform::default())).id();
        (world, player)
    }

    fn run_aim(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(aim);
        schedule.run(world);
    }

    #[test]
    fn aims_at_cursor_on_the_right() {
        // Cursor at (600, 300) maps to world (200, 0) with an identity camera;
        // a player facing right has zero rotation.
        let (mut world, player) = aim_world(Some(Vec2::new(600.0, 300.0)));
        run_aim(&mut world);
        let rotation = world.get::<Transform>(player).unwrap().rotation;
        assert!(rotation.angle_between(Quat::IDENTITY) < 0.01);
    }

    #[test]
    fn aims_at_cursor_above() {
        // Screen Y grows downward: cursor at (400, 100) is above center,
        // mapping to world (0, 200) = straight up = +90°.
        let (mut world, player) = aim_world(Some(Vec2::new(400.0, 100.0)));
        run_aim(&mut world);
        let rotation = world.get::<Transform>(player).unwrap().rotation;
        assert!(rotation.angle_between(Quat::from_rotation_z(FRAC_PI_2)) < 0.01);
    }

    #[test]
    fn keeps_rotation_with_no_cursor() {
        let (mut world, player) = aim_world(None);
        world.get_mut::<Transform>(player).unwrap().rotate_z(FRAC_PI_4);
        run_aim(&mut world);
        let rotation = world.get::<Transform>(player).unwrap().rotation;
        assert!(rotation.angle_between(Quat::from_rotation_z(FRAC_PI_4)) < 0.01);
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
