use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowMode},
};

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
        .add_plugin(VirtualJoystickPlugin)
        .add_startup_system(create_scene)
        .add_system(update_joystick)
        .run();
}

#[derive(Component)]
// Player with velocity
struct Player(pub f32);

fn create_scene(
    mut cmd: Commands,
    asset_server: Res<AssetServer>,
) {
    cmd.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 5.0),
        ..default()
    });
    cmd.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0., 0., 0.),
            ..default()
        },
        texture: asset_server.load("Knob.png"),
        sprite: Sprite {
            color: Color::PURPLE,
            custom_size: Some(Vec2::new(50., 50.)),
            ..default()
        },
        ..default()
    })
    .insert(Player(100.));
    // Spawn Interactable Zone
    cmd.spawn(NodeBundle {
        background_color: BackgroundColor(Color::ORANGE_RED.with_a(0.15)),
        style: Style {
            size: Size {
                width: Val::Percent(100.),
                height: Val::Percent(50.),
            },
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Px(0.),
                bottom: Val::Px(0.),
                ..default()
            },
            ..default()
        },
        ..default()
    })
    // Insert interactable Zone component
    .insert(VirtualJoystickInteractionArea);
    // Spawn Virtual Joystick at horizontal center
    cmd.spawn(
        VirtualJoystickBundle::new(VirtualJoystickNode {
            border_image: asset_server.load("Outline.png"),
            knob_image: asset_server.load("Knob.png"),
            knob_size: Vec2::new(80., 80.),
            dead_zone: 0.,
        })
        .set_color(TintColor(Color::WHITE))
        .set_style(Style {
            size: Size::all(Val::Px(150.)),
            position_type: PositionType::Absolute,
            position: UiRect {
                // Center X position
                left: Val::Percent(35.),
                bottom: Val::Percent(15.),
                ..default()
            },
            ..default()
        }),
    );
}

fn update_joystick(
    mut joystick: EventReader<VirtualJoystickEvent>,
    mut player: Query<(&mut Transform, &Player)>,
    time_step: Res<FixedTime>,
) {
    let (mut player, player_data) = player.single_mut();

    for j in joystick.iter() {
        let Vec2 { x, y } = j.axis();
        player.translation.x += x * player_data.0 * time_step.period.as_secs_f32();
        player.translation.y += y * player_data.0 * time_step.period.as_secs_f32();
    }
}
