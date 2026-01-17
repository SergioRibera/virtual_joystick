use std::sync::Arc;

use bevy::{
    ecs::{
        entity::Entity,
        message::MessageWriter,
        query::With,
        system::{Query, Res, Single},
        world::World,
    },
    input::{mouse::MouseButton, touch::Touches, ButtonInput},
    math::{Rect, Vec2},
    prelude::Children,
    ui::{ComputedNode, Node, PositionType, UiGlobalTransform, Val},
    window::{PrimaryWindow, Window},
};

use crate::{
    components::{
        TouchState, VirtualJoystickInteractionArea, VirtualJoystickState,
        VirtualJoystickUIBackground, VirtualJoystickUIKnob,
    },
    VirtualJoystickID, VirtualJoystickMessage, VirtualJoystickMessageType, VirtualJoystickNode,
};
use bevy::ecs::query::Without;

pub fn update_missing_state<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks = world.query::<(Entity, &VirtualJoystickNode<S>)>();
    let mut joystick_entities: Vec<Entity> = Vec::new();
    for (joystick_entity, _) in joysticks.iter(world) {
        joystick_entities.push(joystick_entity);
    }
    for joystick_entity in joystick_entities {
        let has_state = world.get::<VirtualJoystickState>(joystick_entity).is_some();
        if !has_state {
            world
                .entity_mut(joystick_entity)
                .insert(VirtualJoystickState::default());
        }
    }
}

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
) {
    for (joystick_entity, joystick_node, joystick_global_transform, mut joystick_state) in
        joystick_query
    {
        joystick_state.just_released = false;
        if let Some(touch_state) = &mut joystick_state.touch_state {
            touch_state.just_pressed = false;
        }

        let mut interaction_rect: Option<Rect> = None;
        if let Ok(children) = children_query.get(joystick_entity) {
            for &child in children.iter() {
                if let Ok((node, transform)) = interaction_area_query.get(child) {
                    interaction_rect = Some(Rect::from_center_size(
                        transform.translation * node.inverse_scale_factor(),
                        node.size() * node.inverse_scale_factor(),
                    ));
                    break;
                }
            }
        }

        let interaction_rect = interaction_rect.unwrap_or_else(|| {
            Rect::from_center_size(
                joystick_global_transform.translation * joystick_node.inverse_scale_factor(),
                joystick_node.size() * joystick_node.inverse_scale_factor(),
            )
        });

        if joystick_state.touch_state.is_none() {
            for touch in touches.iter() {
                if interaction_rect.contains(touch.position()) {
                    joystick_state.touch_state = Some(TouchState {
                        id: touch.id(),
                        is_mouse: false,
                        start: touch.position(),
                        current: touch.position(),
                        just_pressed: true,
                    });
                    break;
                }
            }
            if joystick_state.touch_state.is_none() && mouse_buttons.just_pressed(MouseButton::Left)
            {
                if let Some(mouse_pos) = window.cursor_position() {
                    if interaction_rect.contains(mouse_pos) {
                        joystick_state.touch_state = Some(TouchState {
                            id: 0,
                            is_mouse: true,
                            start: mouse_pos,
                            current: mouse_pos,
                            just_pressed: true,
                        });
                    }
                }
            }
        } else {
            let mut clear_touch_state = false;
            if let Some(touch_state) = &joystick_state.touch_state {
                if touch_state.is_mouse {
                    if mouse_buttons.just_released(MouseButton::Left) {
                        clear_touch_state = true;
                    }
                } else if touches.just_released(touch_state.id) {
                    clear_touch_state = true;
                }
            }
            if clear_touch_state {
                joystick_state.touch_state = None;
                joystick_state.just_released = true;
            } else if let Some(touch_state) = &mut joystick_state.touch_state {
                if touch_state.is_mouse {
                    if let Some(new_current) = window.cursor_position() {
                        if new_current != touch_state.current {
                            touch_state.current = new_current;
                        }
                    }
                } else if let Some(touch) = touches.get_pressed(touch_state.id) {
                    let touch_position = touch.position();
                    if touch_position != touch_state.current {
                        touch_state.current = touch_position;
                    }
                }
            }
        }
    }
}

pub fn update_behavior_knob_delta<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks = world.query::<(Entity, &VirtualJoystickNode<S>)>();
    let mut joystick_entities: Vec<Entity> = Vec::new();
    for (joystick_entity, _) in joysticks.iter(world) {
        joystick_entities.push(joystick_entity);
    }
    for joystick_entity in joystick_entities {
        let behavior;
        {
            let Some(virtual_joystick_node) = world.get::<VirtualJoystickNode<S>>(joystick_entity)
            else {
                continue;
            };
            behavior = Arc::clone(&virtual_joystick_node.behavior);
        }
        behavior.update_at_delta_stage(world, joystick_entity);
    }
}

pub fn update_behavior_constraints<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks = world.query::<(Entity, &VirtualJoystickNode<S>)>();
    let mut joystick_entities: Vec<Entity> = Vec::new();
    for (joystick_entity, _) in joysticks.iter(world) {
        joystick_entities.push(joystick_entity);
    }
    for joystick_entity in joystick_entities {
        let behavior;
        {
            let Some(virtual_joystick_node) = world.get::<VirtualJoystickNode<S>>(joystick_entity)
            else {
                continue;
            };
            behavior = Arc::clone(&virtual_joystick_node.behavior);
        }
        behavior.update_at_constraint_stage(world, joystick_entity);
    }
}

pub fn update_behavior<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks = world.query::<(Entity, &VirtualJoystickNode<S>)>();
    let mut joystick_entities: Vec<Entity> = Vec::new();
    for (joystick_entity, _) in joysticks.iter(world) {
        joystick_entities.push(joystick_entity);
    }
    for joystick_entity in joystick_entities {
        let behavior;
        {
            let Some(virtual_joystick_node) = world.get::<VirtualJoystickNode<S>>(joystick_entity)
            else {
                continue;
            };
            behavior = Arc::clone(&virtual_joystick_node.behavior);
        }
        behavior.update(world, joystick_entity);
    }
}

pub fn update_action<S: VirtualJoystickID>(world: &mut World) {
    let mut joysticks =
        world.query::<(Entity, &VirtualJoystickNode<S>, &mut VirtualJoystickState)>();
    let mut joystick_entities: Vec<Entity> = Vec::new();
    for (joystick_entity, _, _) in joysticks.iter(world) {
        joystick_entities.push(joystick_entity);
    }
    enum DragAction {
        StartDrag,
        Drag,
        EndDrag,
    }
    for joystick_entity in joystick_entities {
        let drag_action: Option<DragAction>;
        {
            let Some(joystick_state) = world.get::<VirtualJoystickState>(joystick_entity) else {
                continue;
            };
            if joystick_state.just_released {
                drag_action = Some(DragAction::EndDrag);
            } else if let Some(touch_state) = &joystick_state.touch_state {
                if touch_state.just_pressed {
                    drag_action = Some(DragAction::StartDrag);
                } else {
                    drag_action = Some(DragAction::Drag);
                }
            } else {
                drag_action = None;
            }
        }
        let Some(drag_action) = drag_action else {
            continue;
        };
        let id;
        let action;
        let joystick_state;
        {
            let Ok((_, virtual_joystick_node, joystick_state_2)) =
                joysticks.get_mut(world, joystick_entity)
            else {
                continue;
            };
            id = virtual_joystick_node.id.clone();
            action = Arc::clone(&virtual_joystick_node.action);
            joystick_state = joystick_state_2.clone();
        }
        match drag_action {
            DragAction::StartDrag => {
                action.on_start_drag(id, joystick_state, world, joystick_entity);
            }
            DragAction::Drag => {
                action.on_drag(id, joystick_state, world, joystick_entity);
            }
            DragAction::EndDrag => {
                action.on_end_drag(id, joystick_state, world, joystick_entity);
            }
        }
    }
}

pub fn update_send_messages<S: VirtualJoystickID>(
    joystick_query: Query<(&VirtualJoystickNode<S>, &VirtualJoystickState)>,
    mut writer: MessageWriter<VirtualJoystickMessage<S>>,
) {
    for (joystick, joystick_state) in joystick_query {
        if joystick_state.just_released {
            writer.write(VirtualJoystickMessage {
                id: joystick.id.clone(),
                message_type: VirtualJoystickMessageType::Up,
                value: Vec2::ZERO,
                delta: joystick_state.delta,
            });
            continue;
        }
        if let Some(touch_state) = &joystick_state.touch_state {
            if touch_state.just_pressed {
                writer.write(VirtualJoystickMessage {
                    id: joystick.id.clone(),
                    message_type: VirtualJoystickMessageType::Press,
                    value: touch_state.current,
                    delta: joystick_state.delta,
                });
            }
            writer.write(VirtualJoystickMessage {
                id: joystick.id.clone(),
                message_type: VirtualJoystickMessageType::Drag,
                value: touch_state.current,
                delta: joystick_state.delta,
            });
        }
    }
}

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
        let mut joystick_base_rect: Option<Rect> = None;
        for child in children.iter() {
            if joystick_base_query.contains(*child) {
                let (mut joystick_base_style, joystick_base_node, joystick_base_global_transform) =
                    joystick_base_query.get_mut(*child).unwrap();
                joystick_base_style.position_type = PositionType::Absolute;
                joystick_base_style.left = Val::Px(joystick_state.base_offset.x);
                joystick_base_style.top = Val::Px(joystick_state.base_offset.y);

                let rect = Rect::from_center_size(
                    joystick_base_global_transform.translation
                        * joystick_base_node.inverse_scale_factor,
                    joystick_base_node.size() * joystick_base_node.inverse_scale_factor,
                );
                joystick_base_rect = Some(rect);
            }
        }
        if joystick_base_rect.is_none() {
            continue;
        }
        let joystick_base_rect = joystick_base_rect.unwrap();
        let joystick_base_rect_half_size = joystick_base_rect.half_size();
        for child in children.iter() {
            if joystick_knob_query.contains(*child) {
                let (mut joystick_knob_style, joystick_knob_node, joystick_knob_global_transform) =
                    joystick_knob_query.get_mut(*child).unwrap();
                let joystick_knob_rect = Rect::from_center_size(
                    joystick_knob_global_transform.translation
                        * joystick_knob_node.inverse_scale_factor,
                    joystick_knob_node.size() * joystick_knob_node.inverse_scale_factor,
                );
                let joystick_knob_half_size = joystick_knob_rect.half_size();
                joystick_knob_style.position_type = PositionType::Absolute;
                joystick_knob_style.left = Val::Px(
                    joystick_state.base_offset.x
                        + joystick_base_rect_half_size.x
                        + joystick_knob_half_size.x
                        + (joystick_state.delta.x - 1.0) * joystick_base_rect_half_size.x,
                );
                joystick_knob_style.top = Val::Px(
                    joystick_state.base_offset.y
                        + joystick_base_rect_half_size.y
                        + joystick_knob_half_size.y
                        + (-joystick_state.delta.y - 1.0) * joystick_base_rect_half_size.y,
                );
            }
        }
    }
}
