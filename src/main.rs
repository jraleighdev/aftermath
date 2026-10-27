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
        .add_systems(Update, player::movement)
        .add_systems(Update, player::update_facing)
        .run();
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let down = assets.load("sprites/player_down.png");
    let up = assets.load("sprites/player_up.png");
    let side = assets.load("sprites/player_side.png");
    commands.spawn(Camera2d);
    commands.insert_resource(player::PlayerSprites {
        down: down.clone(),
        up,
        side: side.clone(),
    });
    commands.spawn((
        player::Player,
        shoot::ShotCooldown { remaining: 0.0 },
        player::PlayerFacing::default(),
        Sprite::from_image(down),
        Transform::from_scale(Vec3::splat(6.0)),
    ));
}
