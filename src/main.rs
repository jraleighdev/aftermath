use bevy::prelude::*;
use bevy::window::WindowPlugin;

mod player;
mod shoot;
mod enemies;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Aftermath".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (enemies::spawn_enemies, enemies::chase, enemies::shove_player).chain())
        .add_systems(Update, (shoot::cool_down, shoot::shoot).chain())
        .add_systems(Update, (shoot::move_bullets, shoot::despawn_offscreen, shoot::collide))
        .add_systems(Update, player::update_facing)
        .add_systems(Update, player::movement)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        player::Player,
        shoot::ShotCooldown { remaining: 0.0 },
        Sprite::from_color(Color::srgb(0.2, 0.8, 0.2), Vec2::splat(100.0)),
    ));
}
