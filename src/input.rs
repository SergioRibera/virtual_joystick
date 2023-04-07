use bevy::{input::touch::TouchPhase, prelude::*, ui::RelativeCursorPosition};

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
        for (_node, mut knob) in joysticks.iter_mut() {
            match e.phase {
                TouchPhase::Started => {
                    if knob.interactable_zone_rect.contains(e.position) {
                        knob.id_drag = Some(e.id);
                        knob.start_pos = e.position;
                        knob.current_pos = axis.handle(e.position);
                    }
                }
                TouchPhase::Moved => {
                    if let Some(id) = knob.id_drag {
                        if e.id == id {
                            j_type.handle(&mut knob, e.position);
                            knob.current_pos = axis.handle(e.position);
                            knob.delta = axis.handle(knob.start_pos - e.position);
                        }
                    }
                }
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

            if knob.delta.x.abs() > knob.dead_zone || knob.delta.y.abs() > knob.dead_zone {
                send_values.send(VirtualJoystickEvent {
                    value: knob.current_pos,
                    delta: knob.delta,
                    axis: *axis,
                });
            }
        }
    }
}

// pub fn elastic_out(t: fxx) -> fxx {
//     fxx::sin(-13.0 * (t + 1.0) * FRAC_PI_2) * fxx::powf(2.0, -10.0 * t) + 1.0
// }

pub fn update_joystick_by_mouse(
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_evr: EventReader<CursorMoved>,
    mut send_values: EventWriter<VirtualJoystickEvent>,
    mut joysticks: Query<(
        &VirtualJoystickNode,
        &RelativeCursorPosition,
        &mut VirtualJoystickKnob,
    )>,
    axis: Res<VirtualJoystickAxis>,
    j_type: Res<VirtualJoystickType>,
    windows: Query<&Window>,
) {
    let window = windows.single();
    for (_node, r_pos, mut knob) in joysticks.iter_mut() {
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
            if mouse_button_input.just_pressed(MouseButton::Left)
                && knob.id_drag.is_none()
                && knob.interactable_zone_rect.contains(pos)
            {
                knob.id_drag = Some(0);
                knob.start_pos = pos;
            }

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

        if knob.delta.x.abs() > knob.dead_zone || knob.delta.y.abs() > knob.dead_zone {
            send_values.send(VirtualJoystickEvent {
                value: axis.handle_xy(-knob.current_pos.x, knob.current_pos.y),
                delta: axis.handle_xy(-knob.delta.x, knob.delta.y),
                axis: *axis,
            });
        }
    }
}
