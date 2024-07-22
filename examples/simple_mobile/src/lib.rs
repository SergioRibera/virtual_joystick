use bevy::{prelude::*, window::WindowMode};

use virtual_joystick::*;

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resizable: false,
                mode: WindowMode::Fullscreen,
                title: "Simple Joystick".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(VirtualJoystickPlugin::<String>::default())
        .add_systems(Startup, create_scene)
        .add_systems(Update, update_joystick)
        .run();
}

#[derive(Component)]
// Player with velocity
struct Player(pub f32);

fn create_scene(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 5.0),
        ..default()
    });
    // Fake Player
    cmd.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0., 0., 0.),
            ..default()
        },
        texture: asset_server.load("Knob.png"),
        sprite: Sprite {
            color: Color::srgb(0.5, 0.0, 0.5),
            custom_size: Some(Vec2::new(50., 50.)),
            ..default()
        },
        ..default()
    })
    .insert(Player(50.));

    // Spawn Virtual Joystick at horizontal center using helper function
    create_joystick(
        &mut cmd,
        "UniqueJoystick".to_string(),
        asset_server.load("Knob.png"),
        asset_server.load("Outline.png"),
        None,
        None,
        Some(Color::srgba(1.0, 0.27, 0.0, 0.3)),
        Vec2::new(75., 75.),
        Vec2::new(150., 150.),
        Style {
            width: Val::Px(150.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.),
            bottom: Val::Percent(15.),
            ..default()
        },
        JoystickFloating,
        NoAction,
    );
}

fn update_joystick(
    mut joystick: EventReader<VirtualJoystickEvent<String>>,
    mut player: Query<(&mut Transform, &Player)>,
    time_step: Res<Time>,
) {
    let (mut player, player_data) = player.single_mut();

    for j in joystick.read() {
        let Vec2 { x, y } = j.axis();
        player.translation.x += x * player_data.0 * time_step.delta_seconds();
        player.translation.y += y * player_data.0 * time_step.delta_seconds();
    }
}
