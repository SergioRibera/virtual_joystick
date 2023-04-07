use bevy::{input::touch::TouchPhase, prelude::*};

use crate::{
    joystick::VirtualJoystickKnob, VirtualJoystickAxis, VirtualJoystickEvent, VirtualJoystickNode,
    VirtualJoystickType,
};

pub fn run_if_touch(touch_events: EventReader<TouchInput>) -> bool {
    !touch_events.is_empty()
}

pub fn run_if_pc() -> bool {
    !["android", "ios"].contains(&std::env::consts::OS)
}

pub fn update_joystick(
    mut touch_events: EventReader<TouchInput>,
    mut send_values: EventWriter<VirtualJoystickEvent>,
    mut joysticks: Query<(&VirtualJoystickNode, &mut VirtualJoystickKnob)>,
    axis: Res<VirtualJoystickAxis>,
    j_type: Res<VirtualJoystickType>,
) {
    for e in touch_events.iter() {
        let pos = e.position;
        for (_node, mut knob) in joysticks.iter_mut() {
            match e.phase {
                // Start drag
                TouchPhase::Started => {
                    if knob.interactable_zone_rect.contains(pos) && knob.id_drag.is_none() {
                        knob.id_drag = Some(e.id);
                        knob.start_pos = pos;
                    }
                }
                // Dragging
                TouchPhase::Moved => {
                    if let Some(id) = knob.id_drag {
                        if e.id == id {
                            if *j_type == VirtualJoystickType::Dynamic {
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
                            knob.start_pos = Vec2::ZERO;
                            knob.current_pos = Vec2::ZERO;
                            knob.delta = Vec2::ZERO;
                        }
                    }
                }
            }

            // Send event
            if knob.delta.x.abs() > knob.dead_zone || knob.delta.y.abs() > knob.dead_zone {
                send_values.send(VirtualJoystickEvent {
                    value: axis.handle(knob.current_pos),
                    delta: axis.handle(knob.delta),
                    axis: *axis,
                });
            }
        }
    }
}

pub fn update_joystick_by_mouse(
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_evr: EventReader<CursorMoved>,
    mut send_values: EventWriter<VirtualJoystickEvent>,
    mut joysticks: Query<(
        &VirtualJoystickNode,
        &mut VirtualJoystickKnob,
    )>,
    axis: Res<VirtualJoystickAxis>,
    j_type: Res<VirtualJoystickType>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    for (_node, mut knob) in joysticks.iter_mut() {
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
                        if *j_type == VirtualJoystickType::Dynamic {
                            knob.base_pos = pos;
                        }
                        knob.current_pos = pos;
                        knob.delta = (knob.start_pos - knob.current_pos).normalize_or_zero();
                    }
                }
            }
        }

        // Send event
        if knob.delta.x.abs() > knob.dead_zone || knob.delta.y.abs() > knob.dead_zone {
            send_values.send(VirtualJoystickEvent {
                value: axis.handle_xy(-knob.current_pos.x, knob.current_pos.y),
                delta: axis.handle_xy(-knob.delta.x, knob.delta.y),
                axis: *axis,
            });
        }
    }
}
