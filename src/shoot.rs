use bevy::prelude::*;

use crate::enemies::{Enemy, RADIUS as ENEMY_RADIUS};
use crate::player::{Player, PlayerFacing};
/// A projectile: pixels per second it travels along its velocity.
#[derive(Component)]
pub struct Bullet {
    pub velocity: Vec2,
}

/// Bullet speed in pixels per second.
pub const SPEED: f32 = 600.0;

/// Collision radius: half the bullet sprite's width.
pub const RADIUS: f32 = 6.0;

/// Minimum seconds between shots while holding fire.
pub const SHOT_COOLDOWN: f32 = 0.2;

/// Per-player time remaining before the next shot may fire.
#[derive(Component)]
pub struct ShotCooldown {
    pub remaining: f32,
}

/// Counts every `ShotCooldown` down by the frame's delta.
pub fn cool_down(time: Res<Time>, mut cooldowns: Query<&mut ShotCooldown>) {
    for mut cooldown in &mut cooldowns {
        cooldown.remaining -= time.delta_secs();
    }
}

/// Spawns a bullet from every `Player` along its aim direction while the
/// left mouse button is held (autofire covers click and hold), at most
/// once per `SHOT_COOLDOWN`.
pub fn shoot(
    mouse: Res<ButtonInput<MouseButton>>,
    mut players: Query<(&PlayerFacing, &Transform, &mut ShotCooldown), With<Player>>,
    mut commands: Commands,
) {
    if !mouse.pressed(MouseButton::Left) {
        return;
    }
    for (facing, transform, mut cooldown) in &mut players {
        if cooldown.remaining > 0.0 {
            continue;
        }
        let direction = facing.aim;
        commands.spawn((
            Bullet { velocity: direction * SPEED },
            Transform::from_translation(transform.translation)
                .with_rotation(Quat::from_rotation_z(direction.to_angle())),
            Sprite::from_color(Color::srgb(1.0, 0.2, 0.2), Vec2::new(12.0, 4.0)),
        ));
        cooldown.remaining = SHOT_COOLDOWN;
    }
}

/// Kills the first enemy each bullet touches (and the bullet with it).
pub fn collide(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
) {
    for (bullet, bullet_transform) in &bullets {
        let bullet_pos = bullet_transform.translation.xy();
        let hit = enemies
            .iter()
            .find(|(_, enemy_transform)| {
                bullet_pos.distance(enemy_transform.translation.xy())
                    <= RADIUS + ENEMY_RADIUS
            });
        if let Some((enemy, _)) = hit {
            commands.entity(bullet).despawn();
            commands.entity(enemy).despawn();
        }
    }
}

/// Moves every bullet along its velocity.
pub fn move_bullets(time: Res<Time>, mut bullets: Query<(&Bullet, &mut Transform)>) {
    for (bullet, mut transform) in &mut bullets {
        transform.translation += bullet.velocity.extend(0.0) * time.delta_secs();
    }
}

/// Despawns every bullet that has left the camera's visible world rect.
/// shortcut: assumes an unrotated camera and scale factor 1 (our setup),
/// upgrade when the camera rotates or a UI scale factor appears.
pub fn despawn_offscreen(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform), With<Bullet>>,
    camera: Single<&GlobalTransform, With<Camera>>,
    window: Single<&Window>,
) {
    let half_size = Vec2::new(window.width(), window.height()) * 0.5;
    let camera_pos = camera.translation().xy();
    for (entity, transform) in &bullets {
        // cursor->world showed world = screen - center, so screen = world + center here.
        let screen = transform.translation.xy() - camera_pos + half_size;
        if screen.x < 0.0
            || screen.y < 0.0
            || screen.x > window.width()
            || screen.y > window.height()
        {
            commands.entity(entity).despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enemies::Enemy;
    use crate::player::Facing;
    use bevy::camera::RenderTargetInfo;
    use bevy::window::WindowResolution;
    use std::time::Duration;

    /// World with an 800x600 window and identity camera, one player, and one
    /// bullet at `bullet_pos`, for the despawn tests.
    fn view_world(bullet_pos: Vec2) -> (World, Entity) {
        let mut world = World::default();
        world.insert_resource(Time::<()>::default());

        let mut window = Window {
            resolution: WindowResolution::new(800, 600),
            ..default()
        };
        window.set_cursor_position(None);
        world.spawn(window);

        let camera = world
            .spawn((Camera2d, GlobalTransform::default()))
            .id();
        world.get_mut::<Camera>(camera).unwrap().computed.target_info =
            Some(RenderTargetInfo {
                physical_size: UVec2::new(800, 600),
                scale_factor: 1.0,
            });

        let bullet = world
            .spawn((
                Bullet { velocity: Vec2::X * SPEED },
                Transform::from_translation(bullet_pos.extend(0.0)),
            ))
            .id();
        (world, bullet)
    }

    fn add_despawn(schedule: &mut Schedule) {
        schedule.add_systems(despawn_offscreen);
    }

    fn add_run(schedule: &mut Schedule) {
        schedule.add_systems((cool_down, shoot).chain());
    }

    #[test]
    fn keeps_bullet_on_screen() {
        // Identity camera: the visible world rect is centered on the camera
        // ([-400, -300]..[400, 300]), like cursor->world math showed.
        let (mut world, bullet) = view_world(Vec2::new(100.0, -100.0));
        run(&mut world, add_despawn);
        assert!(world.get_entity(bullet).is_ok());
    }

    #[test]
    fn despawns_bullet_off_screen() {
        let (mut world, bullet) = view_world(Vec2::new(2000.0, 300.0));
        run(&mut world, add_despawn);
        assert!(world.get_entity(bullet).is_err());
    }

    /// World with a 1s-old clock, one player at the origin aiming along
    /// `aim`, and the left mouse button pressed iff `pressed`.
    fn shoot_world(pressed: bool, aim: Vec2) -> World {
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
            ShotCooldown { remaining: 0.0 },
            PlayerFacing { facing: Facing::default(), aim },
            Transform::default(),
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
        // Aim +X (click or hold): bullet flies right.
        let mut world = shoot_world(true, Vec2::X);
        run(&mut world, add_shoot);
        assert_eq!(bullet_count(&mut world), 1);

        let query = world.query::<(&Bullet, &Transform)>().single(&world).unwrap();
        assert!(query.0.velocity.abs_diff_eq(Vec2::new(SPEED, 0.0), 0.01));
        assert!(query.1.translation.xy().abs_diff_eq(Vec2::ZERO, 0.01));
    }

    #[test]
    fn shoots_toward_aim_when_facing_up() {
        // Aim +Y: bullet flies up. Held input counts too (click/hold).
        let mut world = shoot_world(true, Vec2::Y);
        run(&mut world, add_shoot);
        let query = world.query::<(&Bullet, &Transform)>().single(&world).unwrap();
        assert!(query.0.velocity.abs_diff_eq(Vec2::new(0.0, SPEED), 0.01));
    }

    #[test]
    fn shoots_nothing_without_input() {
        let mut world = shoot_world(false, Vec2::X);
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

    /// While holding fire, a second frame inside the cooldown window shoots
    /// nothing — one bullet per window.
    #[test]
    fn shoots_once_within_cooldown_window() {
        let mut world = shoot_world(true, Vec2::X);
        run(&mut world, add_run);
        assert_eq!(bullet_count(&mut world), 1);

        // Next frame arrives 50ms later, still inside the 200ms cooldown.
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(50));
        run(&mut world, add_run);
        assert_eq!(bullet_count(&mut world), 1);
    }

    /// After enough time passes, holding fire shoots again.
    #[test]
    fn shoots_again_after_cooldown_window() {
        let mut world = shoot_world(true, Vec2::X);
        run(&mut world, add_run);
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis((SHOT_COOLDOWN * 1000.0) as u64));
        run(&mut world, add_run);
        assert_eq!(bullet_count(&mut world), 2);
    }

    /// World with one bullet and one enemy at the given positions, plus a
    /// `HitEntities` resource so the assertions can check survival directly.
    fn bullet_enemy_world(bullet_pos: Vec2, enemy_pos: Vec2) -> World {
        let mut world = World::default();
        world.insert_resource(Time::<()>::default());
        let bullet = world
            .spawn((Bullet { velocity: Vec2::ZERO }, Transform::from_translation(bullet_pos.extend(0.0))))
            .id();
        let enemy = world
            .spawn((Enemy, Transform::from_translation(enemy_pos.extend(0.0))))
            .id();
        world.insert_resource(HitEntities { bullet, enemy });
        world
    }

    /// Holds the spawned entities so tests can assert on survival directly.
    #[derive(Resource)]
    struct HitEntities {
        bullet: Entity,
        enemy: Entity,
    }

    fn add_collide(schedule: &mut Schedule) {
        schedule.add_systems(collide);
    }

    #[test]
    fn bullet_kills_itself_and_its_target_enemy() {
        let mut world = bullet_enemy_world(Vec2::ZERO, Vec2::new(35.0, 0.0));
        run(&mut world, add_collide);
        let hit = world.resource::<HitEntities>();
        assert!(world.get_entity(hit.bullet).is_err());
        assert!(world.get_entity(hit.enemy).is_err());
    }

    #[test]
    fn bullet_past_the_hit_range_kills_nothing() {
        let mut world = bullet_enemy_world(Vec2::ZERO, Vec2::new(45.0, 0.0));
        run(&mut world, add_collide);
        let hit = world.resource::<HitEntities>();
        assert!(world.get_entity(hit.bullet).is_ok());
        assert!(world.get_entity(hit.enemy).is_ok());
    }

    #[test]
    fn one_bullet_only_consumes_one_enemy() {
        let mut world = bullet_enemy_world(Vec2::ZERO, Vec2::new(35.0, 0.0));
        let second = world.spawn((Enemy, Transform::default())).id();
        run(&mut world, add_collide);
        assert!(world.get_entity(second).is_ok());
    }
}
