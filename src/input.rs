use bevy::{input::touch::TouchPhase, prelude::*};

use crate::{
    joystick::VirtualJoystickKnob, VirtualJoystickAxis, VirtualJoystickEvent, VirtualJoystickNode,
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
) {
    for e in touch_events.iter() {
        for (_node, mut knob) in joysticks.iter_mut() {
            match e.phase {
                TouchPhase::Started => {
                    if knob.interactable_zone_rect.contains(e.position)
                        && !knob.dead_zone_rect.contains(e.position)
                    {
                        knob.id_drag = Some(e.id);
                        knob.start_pos = e.position;
                        knob.current_pos = axis.handle(e.position);
                    }
                }
                TouchPhase::Moved => {
                    if let Some(id) = knob.id_drag {
                        if e.id == id {
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
            send_values.send(VirtualJoystickEvent {
                value: knob.current_pos,
                delta: knob.delta,
                axis: *axis,
            });
        }
    }
}

pub fn update_joystick_by_mouse(
    mouse_button_input: Res<Input<MouseButton>>,
    mut cursor_evr: EventReader<CursorMoved>,
    mut send_values: EventWriter<VirtualJoystickEvent>,
    mut joysticks: Query<(&VirtualJoystickNode, &mut VirtualJoystickKnob)>,
    axis: Res<VirtualJoystickAxis>,
) {
    for (_node, mut knob) in joysticks.iter_mut() {
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
            if mouse_button_input.just_pressed(MouseButton::Left) && knob.id_drag.is_none() {
                knob.id_drag = Some(0);
                knob.start_pos = e.position;
            }

            if mouse_button_input.pressed(MouseButton::Left) {
                if let Some(id) = knob.id_drag {
                    if 0 == id {
                        knob.current_pos = axis.handle(e.position);
                        knob.delta = axis.handle(knob.start_pos - e.position);
                    }
                }
            }
        }
        send_values.send(VirtualJoystickEvent {
            value: knob.current_pos,
            delta: knob.delta,
            axis: *axis,
        });
    }
}
