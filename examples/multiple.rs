use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use virtual_joystick::*;

/// ID for joysticks
#[derive(Default, Debug, Reflect, Hash, Clone, PartialEq, Eq)]
enum JoystickController {
    #[default]
    MovementX,
    MovementY,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(VirtualJoystickPlugin::<JoystickController>::default())
        .add_systems(Startup, create_scene)
        .add_systems(Update, update_joystick)
        .run();
}

#[derive(Component)]
/// Player with velocity
struct Player(pub f32);

fn create_scene(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2d);
    // Fake Player
    cmd.spawn((
        Sprite {
            image: asset_server.load("Knob.png"),
            color: Color::srgb(0.5, 0.0, 0.5), // Purple
            custom_size: Some(Vec2::new(50., 50.)),
            ..default()
        },
        Player(50.),
        Transform::default(),
    ));

    // Spawn Virtual Joystick on left
    create_joystick(
        &mut cmd,
        JoystickController::MovementX,
        asset_server.load("Knob.png"),
        asset_server.load("Horizontal_Outline_Arrows.png"),
        None,
        None,
        Some(Color::srgba(1.0, 0.27, 0.0, 0.3)),
        Vec2::new(75., 75.),
        Vec2::new(150., 150.),
        Node {
            width: Val::Px(150.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            left: Val::Px(35.),
            bottom: Val::Percent(15.),
            ..default()
        },
        (JoystickFixed, JoystickHorizontalOnly),
        NoAction,
    );

    // Spawn Virtual Joystick on Right
    create_joystick(
        &mut cmd,
        JoystickController::MovementY,
        asset_server.load("Knob.png"),
        asset_server.load("Vertical_Outline_Arrows.png"),
        None,
        None,
        Some(Color::srgba(1.0, 0.27, 0.0, 0.3)),
        Vec2::new(75., 75.),
        Vec2::new(150., 150.),
        Node {
            width: Val::Px(150.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            right: Val::Px(35.),
            bottom: Val::Percent(15.),
            ..default()
        },
        (JoystickFixed, JoystickVerticalOnly),
        NoAction,
    );
}

fn update_joystick(
    mut reader: MessageReader<VirtualJoystickMessage<JoystickController>>,
    player: Single<(&mut Transform, &Player)>,
    time_step: Res<Time>,
) {
    let (mut player, player_data) = player.into_inner();

    for joystick in reader.read() {
        let Vec2 { x, y } = joystick.snap_axis(None);

        match joystick.id() {
            JoystickController::MovementX => {
                player.translation.x += x * player_data.0 * time_step.delta_secs();
            }
            JoystickController::MovementY => {
                player.translation.y += y * player_data.0 * time_step.delta_secs();
            }
        }
    }
}
