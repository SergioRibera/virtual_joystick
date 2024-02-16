use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use virtual_joystick::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(VirtualJoystickPlugin::<String>::default())
        .add_systems(Startup, create_scene)
        .add_systems(PreUpdate, update_joystick_visibility)
        .add_systems(Update, update_joystick)
        .run();
}

#[derive(Component)]
// Player with velocity
struct Player(pub f32);

#[derive(Component)]
struct InvisibleJoystick;

const JOYSTICK_BACKGROUND_SIZE: f32 = 150.0;

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
            color: Color::PURPLE,
            custom_size: Some(Vec2::new(50., 50.)),
            ..default()
        },
        ..default()
    })
    .insert(Player(50.));

    // Spawn Invisible Virtual Joystick with entire screen as active area.
    {
        let knob_img = asset_server.load("Knob.png");
        let background_img = asset_server.load("Outline.png");
        let knob_color = Some(*Color::WHITE.set_a(0.0));
        let background_color = Some(*Color::WHITE.set_a(0.0));
        let interactable_area_color = None;
        let knob_size = Vec2::new(75., 75.);
        let background_size = Vec2::new(JOYSTICK_BACKGROUND_SIZE, JOYSTICK_BACKGROUND_SIZE);
        let joystick_node = VirtualJoystickNode {
            dead_zone: 0.,
            id: "UniqueJoystick".to_string(),
            axis: VirtualJoystickAxis::Both,
            behaviour: VirtualJoystickType::Floating,
        };
        let joystick_node_style = Style {
            width: Val::Percent(100.0), // <-- or 50% to use 1/2 screen
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        };
        let mut spawn =
            cmd.spawn((VirtualJoystickBundle::new(joystick_node).set_style(joystick_node_style), InvisibleJoystick));
        let spawn = spawn
            .insert(VirtualJoystickInteractionArea)
            .with_children(|parent| {
                parent.spawn((
                    VirtualJoystickUIKnob,
                    ImageBundle {
                        image: knob_img.into(),
                        style: Style {
                            width: Val::Px(knob_size.x),
                            height: Val::Px(knob_size.y),
                            ..default()
                        },
                        background_color: knob_color.unwrap_or(Color::WHITE).into(),
                        ..default()
                    },
                ));
                parent.spawn((
                    VirtualJoystickUIBackground,
                    ImageBundle {
                        image: background_img.into(),
                        style: Style {
                            width: Val::Px(background_size.x),
                            height: Val::Px(background_size.y),
                            ..default()
                        },
                        background_color: background_color.unwrap_or(Color::WHITE).into(),
                        ..default()
                    },
                ));
            });

        if let Some(c) = interactable_area_color {
            spawn.insert(BackgroundColor(c));
        }
    }

}

fn update_joystick_visibility(
    mut joystick: Query<(&mut Visibility, &mut Style), With<InvisibleJoystick>>,
    mut joystick_knob_ui: Query<&mut BackgroundColor, With<VirtualJoystickUIKnob>>,
    mut joystick_background_ui: Query<&mut BackgroundColor, (With<VirtualJoystickUIBackground>, Without<VirtualJoystickUIKnob>)>,
    q_windows: Query<&Window, (With<PrimaryWindow>, Without<InvisibleJoystick>)>,
    touches: Res<Touches>,
    buttons: Res<Input<MouseButton>>,
) {
    let change_visibility: Option<Visibility>;
    let mut set_location: Option<Vec2> = None;
    if touches.any_just_pressed() || buttons.any_just_pressed([MouseButton::Left]) {
        change_visibility = Some(Visibility::Visible);
        if touches.any_just_pressed() {
            for touch in touches.iter() {
                set_location = Some(touch.position());
                break;
            }
        } else if buttons.any_just_pressed([MouseButton::Left]) {
            if let Some(position) = q_windows.single().cursor_position() {
                set_location = Some(position);
            }
        }
    } else if touches.any_just_released() || touches.any_just_canceled() || buttons.any_just_released([MouseButton::Left]) {
        change_visibility = Some(Visibility::Hidden);
        set_location = None;
    } else {
        change_visibility = None;
        set_location = None;
    }

    if let Some(change_visibility) = change_visibility {
        for (mut joystick_visibility, mut joystick_style) in &mut joystick {
            if change_visibility == Visibility::Visible {
                joystick_style.width = Val::Px(JOYSTICK_BACKGROUND_SIZE);
                joystick_style.height = Val::Px(JOYSTICK_BACKGROUND_SIZE);
                for mut joystick_knob_ui_color in &mut joystick_knob_ui {
                    joystick_knob_ui_color.0.set_a(1.0);
                }
                for mut joystick_background_ui_color in &mut joystick_background_ui {
                    joystick_background_ui_color.0.set_a(1.0);
                }
            } else if change_visibility == Visibility::Hidden {
                joystick_style.left = Val::Px(0.0);
                joystick_style.top = Val::Px(0.0);
                joystick_style.width = Val::Percent(100.0);
                joystick_style.height = Val::Percent(100.0);
                for mut joystick_knob_ui_color in &mut joystick_knob_ui {
                    joystick_knob_ui_color.0.set_a(0.0);
                }
                for mut joystick_background_ui_color in &mut joystick_background_ui {
                    joystick_background_ui_color.0.set_a(0.0);
                }
            }
        }
    }

    if let Some(set_location) = set_location {
        for (_, mut joystick_style) in &mut joystick {
            joystick_style.left = Val::Px(set_location.x - 0.5 * JOYSTICK_BACKGROUND_SIZE);
            joystick_style.top = Val::Px(set_location.y - 0.5 * JOYSTICK_BACKGROUND_SIZE);
        }
    }
}

fn update_joystick(
    mut joystick_events: EventReader<VirtualJoystickEvent<String>>,
    mut player: Query<(&mut Transform, &Player)>,
    time_step: Res<Time>,
) {
    let (mut player, player_data) = player.single_mut();

    for j in joystick_events.read() {
        let Vec2 { x, y } = j.axis();
        player.translation.x += x * player_data.0 * time_step.delta_seconds();
        player.translation.y += y * player_data.0 * time_step.delta_seconds();
    }
}
