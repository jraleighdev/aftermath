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

/// Marks which way the player sprite faces; the side-right image is
/// mirrored in code for left.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum Facing {
    #[default]
    Down,
    Up,
    Side,
}

/// Component holding the player's current facing and world-space aim
/// direction (toward the cursor), kept for shooting.
#[derive(Component, Default)]
pub struct PlayerFacing {
    pub facing: Facing,
    pub aim: Vec2,
}

/// The three loaded player sprites, one per facing.
#[derive(Resource)]
pub struct PlayerSprites {
    pub down: Handle<Image>,
    pub up: Handle<Image>,
    pub side: Handle<Image>,
}

/// Quantizes mouse aim to a facing: the dominant axis of the
/// cursor→player vector wins (screen Y grows downward). Also keeps the
/// aim direction and swaps the player's sprite/flip to match.
pub fn update_facing(
    mut player: Single<(&mut PlayerFacing, &Transform, &mut Sprite), With<Player>>,
    sprites: Option<Res<PlayerSprites>>,
    camera: Single<(&Camera, &GlobalTransform)>,
    window: Single<&Window>,
) {
    let (camera, camera_transform) = *camera;
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };
    let Ok(cursor_world) = camera.viewport_to_world_2d(camera_transform, cursor_pos) else {
        return;
    };
    let offset = cursor_world - player.1.translation.xy();
    // Viewport Y grows downward but world Y grows upward.
    player.0.facing = if offset.abs().y >= offset.abs().x {
        if offset.y < 0.0 {
            Facing::Down
        } else {
            Facing::Up
        }
    } else {
        Facing::Side
    };
    player.0.aim = offset.normalize_or_zero();
    if let Some(sprites) = sprites {
        let (image, flip_x) = match (player.0.facing, offset.x < 0.0) {
            (Facing::Down, _) => (sprites.down.clone(), false),
            (Facing::Up, _) => (sprites.up.clone(), false),
            (Facing::Side, left) => (sprites.side.clone(), left),
        };
        player.2.image = image;
        player.2.flip_x = flip_x;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::camera::RenderTargetInfo;
    use bevy::window::WindowResolution;
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
        let player = world
            .spawn((Player, PlayerFacing::default(), Transform::default(), Sprite::default()))
            .id();
        (world, player)
    }

    fn run_aim(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(update_facing);
        schedule.run(world);
    }

    #[test]
    fn cursor_below_faces_down() {
        let (mut world, player) = aim_world(Some(Vec2::new(400.0, 500.0)));
        run_aim(&mut world);
        assert_eq!(world.get::<PlayerFacing>(player).unwrap().facing, Facing::Down);
    }

    #[test]
    fn cursor_above_faces_up() {
        let (mut world, player) = aim_world(Some(Vec2::new(400.0, 100.0)));
        run_aim(&mut world);
        assert_eq!(world.get::<PlayerFacing>(player).unwrap().facing, Facing::Up);
    }

    #[test]
    fn cursor_right_faces_side() {
        let (mut world, player) = aim_world(Some(Vec2::new(600.0, 300.0)));
        run_aim(&mut world);
        assert_eq!(world.get::<PlayerFacing>(player).unwrap().facing, Facing::Side);
    }

    #[test]
    fn cursor_left_faces_side() {
        let (mut world, player) = aim_world(Some(Vec2::new(200.0, 300.0)));
        run_aim(&mut world);
        assert_eq!(world.get::<PlayerFacing>(player).unwrap().facing, Facing::Side);
    }

    #[test]
    fn no_cursor_keeps_facing() {
        let (mut world, player) = aim_world(None);
        world.get_mut::<PlayerFacing>(player).unwrap().facing = Facing::Up;
        run_aim(&mut world);
        assert_eq!(world.get::<PlayerFacing>(player).unwrap().facing, Facing::Up);
    }

    /// Distinct dummy image handles for sprite-swap assertions.
    fn insert_sprites(world: &mut World) {
        world.init_resource::<Assets<Image>>();
        let sprites = {
            let mut images = world.resource_mut::<Assets<Image>>();
            PlayerSprites {
                down: images.add(Image::default()),
                up: images.add(Image::default()),
                side: images.add(Image::default()),
            }
        };
        world.insert_resource(sprites);
    }

    #[test]
    fn cursor_left_flips_the_side_sprite() {
        let (mut world, player) = aim_world(Some(Vec2::new(200.0, 300.0)));
        insert_sprites(&mut world);
        run_aim(&mut world);
        let sprite = world.get::<Sprite>(player).unwrap();
        assert!(sprite.flip_x);
    }

    #[test]
    fn cursor_right_keeps_side_sprite_unflipped() {
        let (mut world, player) = aim_world(Some(Vec2::new(600.0, 300.0)));
        insert_sprites(&mut world);
        run_aim(&mut world);
        let sprite = world.get::<Sprite>(player).unwrap();
        assert!(!sprite.flip_x);
    }

    #[test]
    fn cursor_up_swaps_to_the_up_image() {
        let (mut world, player) = aim_world(Some(Vec2::new(400.0, 100.0)));
        insert_sprites(&mut world);
        run_aim(&mut world);
        let sprites = world.resource::<PlayerSprites>();
        let sprite = world.get::<Sprite>(player).unwrap();
        assert_eq!(sprite.image.id(), sprites.up.id());
    }

    #[test]
    fn cursor_below_swaps_to_the_down_image() {
        let (mut world, player) = aim_world(Some(Vec2::new(400.0, 500.0)));
        insert_sprites(&mut world);
        run_aim(&mut world);
        let sprites = world.resource::<PlayerSprites>();
        let sprite = world.get::<Sprite>(player).unwrap();
        assert_eq!(sprite.image.id(), sprites.down.id());
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
