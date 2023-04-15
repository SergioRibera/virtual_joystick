use std::hash::Hash;

use bevy::{input::touch::TouchPhase, prelude::*};

use crate::{
    joystick::VirtualJoystickKnob, VirtualJoystickEvent, VirtualJoystickNode, VirtualJoystickType,
};

pub fn run_if_pc() -> bool {
    !["android", "ios"].contains(&std::env::consts::OS)
}

pub fn update_joystick<S: Hash + Sync + Send + Clone + Default + Reflect + 'static>(
    mut touch_events: EventReader<TouchInput>,
    mut send_values: EventWriter<VirtualJoystickEvent<S>>,
    mut joysticks: Query<(&VirtualJoystickNode<S>, &mut VirtualJoystickKnob)>,
) {
    for (node, mut knob) in joysticks.iter_mut() {
        for e in touch_events.iter() {
            let pos = e.position;
            match e.phase {
                // Start drag
                TouchPhase::Started => {
                    if knob.interactable_zone_rect.contains(pos) && knob.id_drag.is_none() {
                        knob.id_drag = Some(e.id);
                        knob.start_pos = pos;
                        knob.current_pos = pos;
                        knob.delta = Vec2::ZERO;
                    }
                }
                // Dragging
                TouchPhase::Moved => {
                    if let Some(id) = knob.id_drag {
                        if e.id == id {
                            if node.behaviour == VirtualJoystickType::Dynamic {
                                knob.base_pos = pos;
                            }
                            knob.current_pos = pos;
                            knob.delta = (knob.start_pos - knob.current_pos).normalize_or_zero();
                        }
                    }
                }
                // End drag
                TouchPhase::Ended | TouchPhase::Cancelled => {
                    if let Some(id) = knob.id_drag {
                        if e.id == id {
                            knob.id_drag = None;
                            knob.base_pos = Vec2::ZERO;
                            knob.start_pos = Vec2::ZERO;
                            knob.current_pos = Vec2::ZERO;
                            knob.delta = Vec2::ZERO;
                        }
                    }
                }
            }
        }
        // Send event
        if (knob.delta.x.abs() > knob.dead_zone || knob.delta.y.abs() > knob.dead_zone)
            && knob.id_drag.is_some()
        {
            send_values.send(VirtualJoystickEvent {
                id: node.id.clone(),
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
    for (node, mut knob) in joysticks.iter_mut() {
        // End drag
        if mouse_button_input.just_released(MouseButton::Left) {
            if let Some(id) = knob.id_drag {
                if 0 == id {
                    knob.id_drag = None;
                    knob.start_pos = Vec2::ZERO;
                    knob.current_pos = Vec2::ZERO;
                    knob.delta = Vec2::ZERO;
                }
            }
        }

        for e in cursor_evr.iter() {
            let pos = Vec2::new(e.position.x, window.height() - e.position.y);
            // Start drag
            if mouse_button_input.just_pressed(MouseButton::Left)
                && knob.id_drag.is_none()
                && knob.interactable_zone_rect.contains(pos)
            {
                knob.id_drag = Some(0);
                knob.start_pos = pos;
            }

            // Dragging
            if mouse_button_input.pressed(MouseButton::Left) {
                if let Some(id) = knob.id_drag {
                    if 0 == id {
                        if node.behaviour == VirtualJoystickType::Dynamic {
                            knob.base_pos = pos;
                        }
                        knob.current_pos = pos;
                        knob.delta = (knob.start_pos - knob.current_pos).normalize_or_zero();
                    }
                }
            }
        }

        // Send event
        if (knob.delta.x.abs() > knob.dead_zone || knob.delta.y.abs() > knob.dead_zone)
            && knob.id_drag.is_some()
        {
            send_values.send(VirtualJoystickEvent {
                id: node.id.clone(),
                value: node.axis.handle_xy(-knob.current_pos.x, knob.current_pos.y),
                delta: node.axis.handle_xy(-knob.delta.x, knob.delta.y),
                axis: node.axis,
            });
        }
    }
}
