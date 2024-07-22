use bevy::prelude::*;

use crate::{
    VirtualJoystickAction, VirtualJoystickBehavior, VirtualJoystickBundle, VirtualJoystickID,
    VirtualJoystickNode, VirtualJoystickUIBackground, VirtualJoystickUIKnob,
};

/// This function is a simple helper to create a joystick
/// Entity with all needed without complexity
///
/// * `cmd`: bevy Commands, this required to spawn entity
/// * `knob_img`: Handle image for joystick knob
/// * `background_img`: Handle image for joystick border
/// * `knob_color`: Color for joystick knob
/// * `background_color`: Color for joystick border
/// * `interactable_area_color`: Color for interactable zone
/// * `knob_size`: Size for knob ui
/// * `background_size`: Size for joystick border ui
/// * `joystick_node`: [`JoystickNode`] struct
/// * `joystick_node_style`: bevy Style for joystick
///
/// Usage:
/// ```no_run
/// create_joystick(
///     cmd,
///     asset_server.load("Knob.png"),
///     asset_server.load("Outline.png"),
///     None,
///     None,
///     Some(Color::rgba(1.0, 0.27, 0.0, 0.3))),
///     Vec2::new(75., 75.),
///     Vec2::new(150., 150.),
///     VirtualJoystickNode {
///         dead_zone: 0.,
///         id: "UniqueJoystick".to_string(),
///         axis: VirtualJoystickAxis::Both,
///         behaviour: VirtualJoystickType::Floating,
///     },
///     Style {
///         width: Val::Px(150.),
///         height: Val::Px(150.),
///         position_type: PositionType::Absolute,
///         left: Val::Percent(50.),
///         bottom: Val::Percent(15.),
///         ..default()
///     },
/// );
/// ```
///
/// if you not want use this function helper, you need do that
/// ```no_run
/// cmd.spawn((
///     VirtualJoystickBundle::new(VirtualJoystickNode {
///         dead_zone: 0.,
///         id: "UniqueJoystick".to_string(),
///         axis: VirtualJoystickAxis::Both,
///         behaviour: VirtualJoystickType::Floating,
///     })
///     .set_style(Style {
///         width: Val::Px(150.),
///         height: Val::Px(150.),
///         position_type: PositionType::Absolute,
///         left: Val::Percent(50.),
///         bottom: Val::Percent(15.),
///         ..default()
///     }),
///     BackgroundColor(Color::rgba(1.0, 0.27, 0.0, 0.3))),
/// ))
/// .insert(VirtualJoystickInteractionArea)
/// .with_children(|parent| {
///     parent.spawn((
///         VirtualJoystickUIKnob,
///         ImageBundle {
///             image: asset_server.load("Knob.png").into(),
///             style: Style {
///                 width: Val::Px(75.),
///                 height: Val::Px(75.),
///                 ..default()
///             },
///             background_color: Color::WHITE.into(),
///             ..default()
///         },
///     ));
///     parent.spawn((
///         VirtualJoystickUIBackground,
///         ImageBundle {
///             image: asset_server.load("Outline.png").into(),
///             style: Style {
///                 width: Val::Px(150.),
///                 height: Val::Px(150.),
///                 ..default()
///             },
///             background_color: Color::WHITE.into(),
///             ..default()
///         },
///     ));
/// });
/// ```
#[allow(clippy::too_many_arguments)]
pub fn create_joystick<I: VirtualJoystickID>(
    cmd: &mut Commands,
    id: I,
    knob_img: Handle<Image>,
    background_img: Handle<Image>,
    knob_color: Option<Color>,
    background_color: Option<Color>,
    interactable_area_color: Option<Color>,
    knob_size: Vec2,
    background_size: Vec2,
    joystick_node_style: Style,
    behavior: impl VirtualJoystickBehavior,
    action: impl VirtualJoystickAction<I>,
) {
    let mut spawn = cmd.spawn(
        VirtualJoystickBundle::new(
            VirtualJoystickNode::<I>::default()
                .with_id(id)
                .with_behavior(behavior)
                .with_action(action),
        )
        .set_style(joystick_node_style),
    );
    let spawn = spawn.with_children(|parent| {
        parent.spawn((
            VirtualJoystickUIKnob,
            ImageBundle {
                image: knob_img.into(),
                style: Style {
                    position_type: PositionType::Absolute,
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
                    position_type: PositionType::Absolute,
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
