use bevy::{
    input::{mouse::MouseButtonInput, touch::TouchPhase, ButtonState},
    prelude::*,
    window::PrimaryWindow,
};

use crate::VirtualJoystickID;
use crate::{
    ui::VirtualJoystickData, VirtualJoystickEvent, VirtualJoystickEventType, VirtualJoystickNode,
    VirtualJoystickType,
};

#[derive(Event)]
pub enum InputEvent {
    StartDrag { id: u64, pos: Vec2, is_mouse: bool },
    Dragging { id: u64, pos: Vec2, is_mouse: bool },
    EndDrag { id: u64, pos: Vec2, is_mouse: bool },
}

fn is_some_and<T>(opt: Option<T>, cb: impl FnOnce(T) -> bool) -> bool {
    if let Some(v) = opt {
        return cb(v);
    }
    false
}

pub fn update_input<S: VirtualJoystickID>(
    mut input_events: EventReader<InputEvent>,
    mut send_values: EventWriter<VirtualJoystickEvent<S>>,
    mut joysticks: Query<(
        &VirtualJoystickNode<S>,
        &Visibility,
        &InheritedVisibility,
        &ViewVisibility,
        &mut VirtualJoystickData,
    )>,
) {
    let input_events = input_events.read().collect::<Vec<&InputEvent>>();

    for (node, visibility, inherited_visibility, view_visibility, mut knob) in joysticks.iter_mut()
    {
        if visibility == Visibility::Hidden || !inherited_visibility.get() || !view_visibility.get()
        {
            continue;
        }
        for event in &input_events {
            match event {
                InputEvent::StartDrag { id, pos, is_mouse } => {
                    if let Some(current_iteraction_is_mouse) = &knob.current_iteraction_is_mouse {
                        if *current_iteraction_is_mouse != *is_mouse {
                            continue;
                        }
                    }
                    if knob.interactable_zone_rect.contains(*pos) && knob.id_drag.is_none()
                        || is_some_and(knob.id_drag, |i| i != *id)
                            && knob.interactable_zone_rect.contains(*pos)
                    {
                        knob.id_drag = Some(*id);
                        knob.start_pos = *pos;
                        knob.current_pos = *pos;
                        knob.delta = Vec2::ZERO;
                        knob.current_iteraction_is_mouse = Some(*is_mouse);
                        send_values.send(VirtualJoystickEvent {
                            id: node.id.clone(),
                            event: VirtualJoystickEventType::Press,
                            value: Vec2::ZERO,
                            delta: Vec2::ZERO,
                            axis: node.axis,
                        });
                    }
                }
                InputEvent::Dragging { id, pos, is_mouse } => {
                    if let Some(current_iteraction_is_mouse) = &knob.current_iteraction_is_mouse {
                        if *current_iteraction_is_mouse != *is_mouse {
                            continue;
                        }
                    }
                    if !is_some_and(knob.id_drag, |i| i == *id) {
                        continue;
                    }
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
                InputEvent::EndDrag {
                    id,
                    pos: _,
                    is_mouse,
                } => {
                    if let Some(current_iteraction_is_mouse) = &knob.current_iteraction_is_mouse {
                        if *current_iteraction_is_mouse != *is_mouse {
                            continue;
                        }
                    }
                    if !is_some_and(knob.id_drag, |i| i == *id) {
                        continue;
                    }
                    knob.id_drag = None;
                    knob.base_pos = Vec2::ZERO;
                    knob.start_pos = Vec2::ZERO;
                    knob.current_pos = Vec2::ZERO;
                    knob.delta = Vec2::ZERO;
                    knob.current_iteraction_is_mouse = None;
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

pub fn update_joystick(
    mut touch_events: EventReader<TouchInput>,
    mut send_values: EventWriter<InputEvent>,
) {
    let touches = touch_events
        .read()
        .map(|e| (e.id, e.phase, e.position))
        .collect::<Vec<(u64, TouchPhase, Vec2)>>();

    for (id, phase, pos) in &touches {
        match phase {
            // Start drag
            TouchPhase::Started => {
                send_values.send(InputEvent::StartDrag {
                    id: *id,
                    pos: *pos,
                    is_mouse: false,
                });
            }
            // Dragging
            TouchPhase::Moved => {
                send_values.send(InputEvent::Dragging {
                    id: *id,
                    pos: *pos,
                    is_mouse: false,
                });
            }
            // End drag
            TouchPhase::Ended | TouchPhase::Canceled => {
                send_values.send(InputEvent::EndDrag {
                    id: *id,
                    pos: *pos,
                    is_mouse: false,
                });
            }
        }
    }
}

pub fn update_joystick_by_mouse(
    mouse_button_input: Res<Input<MouseButton>>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut send_values: EventWriter<InputEvent>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();
    let pos = window.cursor_position().unwrap_or(Vec2::ZERO);

    for mousebtn in mousebtn_evr.read() {
        // End drag
        if mousebtn.button == MouseButton::Left && mousebtn.state == ButtonState::Released {
            send_values.send(InputEvent::EndDrag {
                id: 0,
                pos,
                is_mouse: true,
            });
        }

        // Start drag
        if mousebtn.button == MouseButton::Left && mousebtn.state == ButtonState::Pressed {
            send_values.send(InputEvent::StartDrag {
                id: 0,
                pos,
                is_mouse: true,
            });
        }
    }

    // Dragging
    if mouse_button_input.pressed(MouseButton::Left) {
        send_values.send(InputEvent::Dragging {
            id: 0,
            pos,
            is_mouse: true,
        });
    }
}
