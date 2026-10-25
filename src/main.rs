use bevy::prelude::*;
use bevy::window::WindowPlugin;

mod player;
mod shoot;

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
        .add_systems(Update, (shoot::shoot, shoot::move_bullets))
        .add_systems(Update, player::aim)
        .add_systems(Update, player::movement)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        player::Player,
        Sprite::from_color(Color::srgb(0.2, 0.8, 0.2), Vec2::splat(100.0)),
    ));
}
