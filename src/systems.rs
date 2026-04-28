use std::sync::Arc;

use bevy::{
    ecs::{
        entity::Entity,
        message::MessageWriter,
        query::With,
        system::{Query, Res, Single},
        world::World,
    },
    input::{ButtonInput, mouse::MouseButton, touch::Touches},
    math::{Rect, Vec2},
    prelude::Children,
    ui::{ComputedNode, Node, PositionType, UiGlobalTransform, UiScale, Val},
    window::{PrimaryWindow, Window},
};

use crate::{
    VirtualJoystickID, VirtualJoystickMessage, VirtualJoystickMessageType, VirtualJoystickNode,
    components::{
        TouchState, VirtualJoystickInteractionArea, VirtualJoystickState,
        VirtualJoystickUIBackground, VirtualJoystickUIKnob,
    },
};
use bevy::ecs::query::Without;

/// Current action being performed by the mouse/touch input
enum DragAction {
    Start,
    Move,
    End,
}

/// Add missing [`VirtualJoystickState`]s for [`Entity`]s with [`VirtualJoystickNode`]
pub fn update_missing_state<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks = world.query_filtered::<Entity, With<VirtualJoystickNode<S>>>();
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    for entity in joysticks {
        if world.get::<VirtualJoystickState>(entity).is_none() {
            world
                .entity_mut(entity)
                .insert(VirtualJoystickState::default());
        }
    }
}

/// Update stored inputs in [`VirtualJoystickState`].
pub fn update_input(
    window: Single<&Window, With<PrimaryWindow>>,
    joystick_query: Query<(
        Entity,
        &ComputedNode,
        &UiGlobalTransform,
        &mut VirtualJoystickState,
    )>,
    children_query: Query<&Children>,
    interaction_area_query: Query<
        (&ComputedNode, &UiGlobalTransform),
        With<VirtualJoystickInteractionArea>,
    >,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    ui_scale: Res<UiScale>,
) {
    for (entity, node, transform, mut state) in joystick_query {
        state.just_released = false;

        // Get interaction rect or fallback to default calculated with `transform` from `joystick_query`.
        let interaction_rect = if let Some(children) = children_query.get(entity).into_iter().next()
            && let Some((node, interaction_transform)) = children
                .iter()
                .find_map(|&child| interaction_area_query.get(child).ok())
        {
            node_rect(node, interaction_transform.translation, ui_scale.0)
        } else {
            node_rect(node, transform.translation, ui_scale.0)
        };

        if let Some(touch_state) = &mut state.touch_state {
            touch_state.just_pressed = false;

            // Either reset `state` input if it has just been released or update with current position.
            if let Some(id) = touch_state.id {
                if touches.just_released(id) {
                    state.reset_input();
                } else if let Some(touch) = touches.get_pressed(id) {
                    touch_state.set_new_current(touch.position());
                }
            } else {
                if mouse_buttons.just_released(MouseButton::Left) {
                    state.reset_input();
                } else if let Some(current) = window.cursor_position() {
                    touch_state.set_new_current(current);
                }
            }
        } else if let Some(touch) = touches
            .iter()
            .find(|touch| interaction_rect.contains(touch.position()))
        {
            // If using touch and within the interaction rect, set `state.touch_state` to touch input.
            state.touch_state = Some(TouchState::new(Some(touch.id()), touch.position()));
        } else if mouse_buttons.just_pressed(MouseButton::Left)
            && let Some(mouse_pos) = window.cursor_position()
            && interaction_rect.contains(mouse_pos)
        {
            // If the left mouse button has just been pressed within the interaction rect,
            // set `state.touch_state` to mouse input.
            state.touch_state = Some(TouchState::new(None, mouse_pos));
        }
    }
}

/// Update behavior knob delta by calling [`crate::behavior::VirtualJoystickBehavior::update_at_delta_stage`] for each joystick entity.
pub fn update_behavior_knob_delta<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks = world.query_filtered::<Entity, With<VirtualJoystickNode<S>>>();
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    for entity in joysticks {
        let behavior = {
            let Some(virtual_joystick_node) = world.get::<VirtualJoystickNode<S>>(entity) else {
                continue;
            };
            Arc::clone(&virtual_joystick_node.behavior)
        };
        behavior.update_at_delta_stage(world, entity);
    }
}

/// Update behavior constraints by calling [`crate::behavior::VirtualJoystickBehavior::update_at_constraint_stage`] for each joystick entity.
pub fn update_behavior_constraints<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks = world.query_filtered::<Entity, With<VirtualJoystickNode<S>>>();
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    for entity in joysticks {
        let behavior = {
            let Some(virtual_joystick_node) = world.get::<VirtualJoystickNode<S>>(entity) else {
                continue;
            };
            Arc::clone(&virtual_joystick_node.behavior)
        };
        behavior.update_at_constraint_stage(world, entity);
    }
}

/// Update behavior by calling [`crate::behavior::VirtualJoystickBehavior::update`] for each joystick entity.
pub fn update_behavior<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks = world.query_filtered::<Entity, With<VirtualJoystickNode<S>>>();
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    for entity in joysticks {
        let behavior = {
            let Some(virtual_joystick_node) = world.get::<VirtualJoystickNode<S>>(entity) else {
                continue;
            };
            Arc::clone(&virtual_joystick_node.behavior)
        };
        behavior.update(world, entity);
    }
}

/// Update [`crate::VirtualJoystickAction`] from [`VirtualJoystickState`].
pub fn update_action<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks =
        world.query::<(Entity, &VirtualJoystickNode<S>, &mut VirtualJoystickState)>();
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    // Collect actions to be executed
    let mut actions = Vec::new();
    for (entity, node, state) in joysticks {
        let Some(joystick_state) = world.get::<VirtualJoystickState>(entity) else {
            continue;
        };
        let drag_action = if joystick_state.just_released {
            DragAction::End
        } else if let Some(touch_state) = &joystick_state.touch_state {
            if touch_state.just_pressed {
                DragAction::Start
            } else {
                DragAction::Move
            }
        } else {
            continue;
        };

        actions.push((
            node.id.clone(),
            Arc::clone(&node.action),
            drag_action,
            state.clone(),
            entity,
        ));
    }
    // Execute appropriate actions for `drag_action`s
    for (id, action, drag_action, state, entity) in actions {
        match drag_action {
            DragAction::Start => {
                action.on_start_drag(id, state, world, entity);
            }
            DragAction::Move => {
                action.on_drag(id, state, world, entity);
            }
            DragAction::End => {
                action.on_end_drag(id, state, world, entity);
            }
        }
    }
}

/// Send [VirtualJoystickMessage]s from [`VirtualJoystickState`].
pub fn update_send_messages<S: VirtualJoystickID>(
    joystick_query: Query<(&VirtualJoystickNode<S>, &VirtualJoystickState)>,
    mut writer: MessageWriter<VirtualJoystickMessage<S>>,
) {
    for (joystick, state) in joystick_query {
        let id = joystick.id.clone();
        let delta = state.delta;
        let Some((message_type, value)) = message_type_and_value(state) else {
            continue;
        };

        writer.write(VirtualJoystickMessage {
            id,
            message_type,
            value,
            delta,
        });
    }
}

/// Update visual representation of the joysticks by interpreting [`VirtualJoystickState`].
#[allow(clippy::complexity)]
pub fn update_ui(
    mut joystick_base_query: Query<
        (&mut Node, &ComputedNode, &UiGlobalTransform),
        With<VirtualJoystickUIBackground>,
    >,
    mut joystick_knob_query: Query<
        (&mut Node, &ComputedNode, &UiGlobalTransform),
        (
            With<VirtualJoystickUIKnob>,
            Without<VirtualJoystickUIBackground>,
        ),
    >,
    joystick_query: Query<(&VirtualJoystickState, &Children)>,
) {
    for (joystick_state, children) in joystick_query {
        let Some(base) = children
            .iter()
            .find(|entity| joystick_base_query.contains(**entity))
        else {
            return;
        };
        let (mut base_style, base_node, base_global_transform) =
            joystick_base_query.get_mut(*base).unwrap();

        // Adjust position of base to match `joystick_state.base_offset`
        base_style.position_type = PositionType::Absolute;
        base_style.left = Val::Px(joystick_state.base_offset.x);
        base_style.top = Val::Px(joystick_state.base_offset.y);

        // NOTE: This uses 1. for `ui_scale` to avoid double scaling since
        //       this is later used in a calculation of a `Node` position.
        let base_rect_half_size =
            node_rect(base_node, base_global_transform.translation, 1.).half_size();

        let Some(knob) = children
            .iter()
            .find(|entity| joystick_knob_query.contains(**entity))
        else {
            return;
        };
        let (mut knob_style, knob_node, knob_global_transform) =
            joystick_knob_query.get_mut(*knob).unwrap();
        // NOTE: This uses 1. for `ui_scale` to avoid double scaling since
        //       this is later used in a calculation of a `Node` position.
        let knob_rect_half_size =
            node_rect(knob_node, knob_global_transform.translation, 1.).half_size();

        // Adjust position of knob to match correct axial movement.
        let delta = joystick_state.delta;
        // NOTE: We are inverting y to align with user intent because `offset` is reversed on the y axis.
        let delta = Vec2::new(delta.x, -delta.y);
        let Vec2 { x, y } = joystick_state.base_offset
            + base_rect_half_size
            + knob_rect_half_size
            + base_rect_half_size * (delta - 1.);
        knob_style.position_type = PositionType::Absolute;
        knob_style.left = Val::Px(x);
        knob_style.top = Val::Px(y);
    }
}

/// The [`Rect`] of a [`ComputedNode`].
fn node_rect(node: &ComputedNode, translation: Vec2, ui_scale: f32) -> Rect {
    let factor = node.inverse_scale_factor * ui_scale;
    Rect::from_center_size(translation * factor, node.size() * factor)
}

/// The appropriate [`VirtualJoystickMessageType`] and the appropriate [`VirtualJoystickMessage::value`] from [`VirtualJoystickState`].
fn message_type_and_value(
    state: &VirtualJoystickState,
) -> Option<(VirtualJoystickMessageType, Vec2)> {
    if state.just_released {
        Some((VirtualJoystickMessageType::Up, Vec2::ZERO))
    } else {
        state.touch_state.as_ref().map(|touch_state| {
            if touch_state.just_pressed {
                (VirtualJoystickMessageType::Press, touch_state.current)
            } else {
                (VirtualJoystickMessageType::Drag, touch_state.current)
            }
        })
    }
}
