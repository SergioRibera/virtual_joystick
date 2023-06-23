use std::hash::Hash;

use bevy::{input::touch::TouchPhase, prelude::*};

use crate::{
    joystick::VirtualJoystickKnob, VirtualJoystickEvent, VirtualJoystickEventType,
    VirtualJoystickNode, VirtualJoystickType,
};

pub fn run_if_pc() -> bool {
    !["android", "ios"].contains(&std::env::consts::OS)
}

fn is_some_and<T>(opt: Option<T>, cb: impl FnOnce(T) -> bool) -> bool {
    if let Some(v) = opt {
        return cb(v);
    }
    false
}

pub fn update_joystick<S: Hash + Sync + Send + Clone + Default + Reflect + 'static>(
    mut touch_events: EventReader<TouchInput>,
    mut send_values: EventWriter<VirtualJoystickEvent<S>>,
    mut joysticks: Query<(&VirtualJoystickNode<S>, &mut VirtualJoystickKnob)>,
) {
    let touches = touch_events
        .iter()
        .map(|e| (e.id, e.phase, e.position))
        .collect::<Vec<(u64, TouchPhase, Vec2)>>();

    for (node, mut knob) in joysticks.iter_mut() {
        for (id, phase, pos) in &touches {
            match phase {
                // Start drag
                TouchPhase::Started => {
                    if knob.interactable_zone_rect.contains(*pos) && knob.id_drag.is_none()
                        || is_some_and(knob.id_drag, |i| i != *id)
                            && knob.interactable_zone_rect.contains(*pos)
                    {
                        knob.id_drag = Some(*id);
                        knob.start_pos = *pos;
                        knob.current_pos = *pos;
                        knob.delta = Vec2::ZERO;
                        send_values.send(VirtualJoystickEvent {
                            id: node.id.clone(),
                            event: VirtualJoystickEventType::Press,
                            value: Vec2::ZERO,
                            delta: Vec2::ZERO,
                            axis: node.axis,
                        });
                    }
                }
                // Dragging
                TouchPhase::Moved => {
                    if is_some_and(knob.id_drag, |i| i == *id) {
                        knob.current_pos = *pos;
                        let half = knob.interactable_zone_rect.half_size();
                        if node.behaviour == VirtualJoystickType::Dynamic {
                            knob.base_pos = *pos;
                            let to_knob = knob.current_pos - knob.start_pos;
                            let distance_to_knob = to_knob.length();
                            if distance_to_knob > half.x {
                                let excess_distance = distance_to_knob - half.x;
                                knob.start_pos += to_knob.normalize() * excess_distance;
                            }
                        }
                        // knob.delta = (knob.start_pos - knob.current_pos).normalize_or_zero();
                        let d = (knob.start_pos - knob.current_pos) / half;
                        knob.delta = Vec2::new(
                            d.x.signum() * d.x.abs().min(1.),
                            d.y.signum() * d.y.abs().min(1.),
                        );
                    }
                }
                // End drag
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if is_some_and(knob.id_drag, |i| i == *id) {
                        knob.id_drag = None;
                        knob.base_pos = Vec2::ZERO;
                        knob.start_pos = Vec2::ZERO;
                        knob.current_pos = Vec2::ZERO;
                        knob.delta = Vec2::ZERO;
                        send_values.send(VirtualJoystickEvent {
                            id: node.id.clone(),
                            event: VirtualJoystickEventType::Up,
                            value: Vec2::ZERO,
                            delta: Vec2::ZERO,
                            axis: node.axis,
                        });
                    }
                }
            }
        }
        // Send event
        if (knob.delta.x.abs() >= knob.dead_zone || knob.delta.y.abs() >= knob.dead_zone)
            && knob.id_drag.is_some()
        {
            send_values.send(VirtualJoystickEvent {
                id: node.id.clone(),
                event: VirtualJoystickEventType::Drag,
                value: node.axis.handle_xy(-knob.current_pos.x, knob.current_pos.y),
                delta: node.axis.handle_xy(-knob.delta.x, knob.delta.y),
                axis: node.axis,
            });
        }
    }
}

pub fn update_joystick_by_mouse<S: Hash + Sync + Send + Clone + Default + Reflect + 'static>(
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_evr: EventReader<CursorMoved>,
    mut send_values: EventWriter<VirtualJoystickEvent<S>>,
    mut joysticks: Query<(&VirtualJoystickNode<S>, &mut VirtualJoystickKnob)>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    let mouse_positions = cursor_evr
        .iter()
        .map(|e| Vec2::new(e.position.x, window.height() - e.position.y))
        .collect::<Vec<Vec2>>();

    for (node, mut knob) in joysticks.iter_mut() {
        // End drag
        if mouse_button_input.just_released(MouseButton::Left)
            && is_some_and(knob.id_drag, |i| i == 0)
        {
            knob.id_drag = None;
            knob.start_pos = Vec2::ZERO;
            knob.current_pos = Vec2::ZERO;
            knob.delta = Vec2::ZERO;
            send_values.send(VirtualJoystickEvent {
                id: node.id.clone(),
                event: VirtualJoystickEventType::Up,
                value: Vec2::ZERO,
                delta: Vec2::ZERO,
                axis: node.axis,
            });
        }

        for pos in &mouse_positions {
            // Start drag
            if mouse_button_input.just_pressed(MouseButton::Left)
                && knob.id_drag.is_none()
                && knob.interactable_zone_rect.contains(*pos)
            {
                knob.id_drag = Some(0);
                knob.start_pos = *pos;
                send_values.send(VirtualJoystickEvent {
                    id: node.id.clone(),
                    event: VirtualJoystickEventType::Press,
                    value: Vec2::ZERO,
                    delta: Vec2::ZERO,
                    axis: node.axis,
                });
            }

            // Dragging
            if mouse_button_input.pressed(MouseButton::Left)
                && is_some_and(knob.id_drag, |i| i == 0)
            {
                knob.current_pos = *pos;
                let half = knob.interactable_zone_rect.half_size();
                if node.behaviour == VirtualJoystickType::Dynamic {
                    knob.base_pos = *pos;
                    let to_knob = knob.current_pos - knob.start_pos;
                    let distance_to_knob = to_knob.length();
                    if distance_to_knob > half.x {
                        let excess_distance = distance_to_knob - half.x;
                        knob.start_pos += to_knob.normalize() * excess_distance;
                    }
                }
                let d = (knob.start_pos - knob.current_pos) / half;
                knob.delta = Vec2::new(
                    d.x.signum() * d.x.abs().min(1.),
                    d.y.signum() * d.y.abs().min(1.),
                );
            }
        }

        // Send event
        if (knob.delta.x.abs() >= knob.dead_zone || knob.delta.y.abs() >= knob.dead_zone)
            && knob.id_drag.is_some()
        {
            send_values.send(VirtualJoystickEvent {
                id: node.id.clone(),
                event: VirtualJoystickEventType::Drag,
                value: node.axis.handle_xy(-knob.current_pos.x, knob.current_pos.y),
                delta: node.axis.handle_xy(-knob.delta.x, knob.delta.y),
                axis: node.axis,
            });
        }
    }
}
