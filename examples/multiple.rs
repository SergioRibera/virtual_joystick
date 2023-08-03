use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use virtual_joystick::*;

// ID for joysticks
#[derive(Default, Reflect, Hash, Clone, PartialEq, Eq)]
enum JoystickController {
    #[default]
    MovementX,
    MovementY,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(VirtualJoystickPlugin::<JoystickController>::default())
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
    .insert(Player(50.));
    // Spawn Virtual Joystick on left
    cmd.spawn(
        VirtualJoystickBundle::new(VirtualJoystickNode {
            border_image: asset_server.load("Horizontal_Outline_Arrows.png"),
            knob_image: asset_server.load("Knob.png"),
            knob_size: Vec2::new(80., 80.),
            dead_zone: 0.,
            id: JoystickController::MovementX,
            axis: VirtualJoystickAxis::Horizontal,
            behaviour: VirtualJoystickType::Fixed,
        })
        .set_color(TintColor(Color::WHITE.with_a(0.2)))
        .set_style(Style {
            width: Val::Px(150.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            left: Val::Px(35.),
            bottom: Val::Percent(15.),
            ..default()
        }),
    )
    .insert(BackgroundColor(Color::ORANGE_RED.with_a(0.2)))
    .insert(VirtualJoystickInteractionArea);

    // Spawn Virtual Joystick on Right
    cmd.spawn(
        VirtualJoystickBundle::new(VirtualJoystickNode {
            border_image: asset_server.load("Vertical_Outline_Arrows.png"),
            knob_image: asset_server.load("Knob.png"),
            knob_size: Vec2::new(80., 80.),
            dead_zone: 0.,
            id: JoystickController::MovementY,
            axis: VirtualJoystickAxis::Vertical,
            behaviour: VirtualJoystickType::Fixed,
        })
        .set_color(TintColor(Color::WHITE.with_a(0.2)))
        .set_style(Style {
            width: Val::Px(150.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            right: Val::Px(35.),
            bottom: Val::Percent(15.),
            ..default()
        }),
    )
    .insert(BackgroundColor(Color::ORANGE_RED.with_a(0.2)))
    .insert(VirtualJoystickInteractionArea);
}

fn update_joystick(
    mut joystick: EventReader<VirtualJoystickEvent<JoystickController>>,
    mut joystick_color: Query<(&mut TintColor, &VirtualJoystickNode<JoystickController>)>,
    mut player: Query<(&mut Transform, &Player)>,
    time_step: Res<FixedTime>,
) {
    let (mut player, player_data) = player.single_mut();

    for j in joystick.iter() {
        let Vec2 { x, y } = j.snap_axis(None);

        match j.get_type() {
            VirtualJoystickEventType::Press | VirtualJoystickEventType::Drag => {
                for (mut color, node) in joystick_color.iter_mut() {
                    if node.id == j.id() {
                        *color = TintColor(Color::WHITE);
                    }
                }
            }
            VirtualJoystickEventType::Up => {
                for (mut color, node) in joystick_color.iter_mut() {
                    if node.id == j.id() {
                        *color = TintColor(Color::WHITE.with_a(0.2));
                    }
                }
            }
        }

        match j.id() {
            JoystickController::MovementX => {
                player.translation.x += x * player_data.0 * time_step.period.as_secs_f32();
            }
            JoystickController::MovementY => {
                player.translation.y += y * player_data.0 * time_step.period.as_secs_f32();
            }
        }
    }
}
