use std::{hash::Hash, marker::PhantomData};

use bevy::{
    prelude::*,
    render::RenderApp,
    ui::{RenderUiSystem, UiSystem}, reflect::TypePath,
};

mod behaviour;
mod input;
mod joystick;

pub use behaviour::{VirtualJoystickAxis, VirtualJoystickType};
use input::{run_if_pc, update_input, update_joystick, update_joystick_by_mouse, InputEvent};
pub use joystick::{
    TintColor, VirtualJoystickBundle, VirtualJoystickInteractionArea, VirtualJoystickNode,
};

use joystick::{extract_joystick_node, VirtualJoystickKnob};

#[derive(Default)]
pub struct VirtualJoystickPlugin<S> {
    _marker: PhantomData<S>,
}

impl<S: Hash + Sync + Send + Clone + Default + Reflect + TypePath + FromReflect + 'static> Plugin
    for VirtualJoystickPlugin<S>
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<TintColor>()
            .register_type::<VirtualJoystickInteractionArea>()
            .register_type::<VirtualJoystickNode<S>>()
            .register_type::<VirtualJoystickKnob>()
            .register_type::<VirtualJoystickAxis>()
            .register_type::<VirtualJoystickType>()
            .register_type::<VirtualJoystickEventType>()
            .add_event::<VirtualJoystickEvent<S>>()
            .add_event::<InputEvent>()
            .add_systems(
                PreUpdate,
                update_joystick.before(update_input::<S>).run_if(not(run_if_pc))
            )
            .add_systems(
                PreUpdate,
                update_joystick_by_mouse.before(update_input::<S>).run_if(run_if_pc))
            .add_systems(
                PreUpdate,
                update_input::<S>
            )
            .add_systems(
                PostUpdate,
                joystick_image_node_system::<S>.before(UiSystem::Layout)
            );

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
        render_app.add_systems(
            ExtractSchedule,
            extract_joystick_node::<S>.after(RenderUiSystem::ExtractNode),
        );
    }
}

fn joystick_image_node_system<S: Hash + Sync + Send + Clone + Default + Reflect + FromReflect + 'static>(
    interaction_area: Query<(&Node, With<VirtualJoystickInteractionArea>)>,
    mut joystick: Query<(
        &Transform,
        &VirtualJoystickNode<S>,
        &mut VirtualJoystickKnob,
    )>,
) {
    let interaction_area = interaction_area
        .iter()
        .map(|(node, _)| node.size())
        .collect::<Vec<Vec2>>();

    for (i, (j_pos, data, mut knob)) in joystick.iter_mut().enumerate() {
        let j_pos = j_pos.translation.truncate();
        let Some(size) = interaction_area.get(i) else {
            return;
        };
        let interaction_area = Rect::from_center_size(j_pos, *size);
        knob.dead_zone = data.dead_zone;
        knob.interactable_zone_rect = interaction_area;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum VirtualJoystickEventType {
    Press,
    Drag,
    Up,
}

#[derive(Event)]
pub struct VirtualJoystickEvent<S: Hash + Sync + Send + Clone + Default + Reflect + 'static + TypePath> {
    id: S,
    event: VirtualJoystickEventType,
    value: Vec2,
    delta: Vec2,
    axis: VirtualJoystickAxis,
}

impl<S: Hash + Sync + Send + Clone + Default + Reflect + TypePath + 'static> VirtualJoystickEvent<S> {
    /// Get ID of joystick throw event
    pub fn id(&self) -> S {
        self.id.clone()
    }
    /// Raw position of point (Mouse or Touch)
    pub fn value(&self) -> Vec2 {
        self.value
    }

    /// Axis of Joystick see [crate::VirtualJoystickAxis]
    pub fn direction(&self) -> VirtualJoystickAxis {
        self.axis
    }

    /// Delta value ranging from 0 to 1 in each vector (x and y)
    pub fn axis(&self) -> Vec2 {
        self.delta
    }

    /// Return the Type of Joystick Event
    pub fn get_type(&self) -> VirtualJoystickEventType {
        self.event
    }

    /// Delta value snaped
    /// the dead_zone is required for make more customizable
    /// the default of the dead_zone is 0.5
    pub fn snap_axis(&self, dead_zone: Option<f32>) -> Vec2 {
        let dead_zone = dead_zone.unwrap_or(0.5);
        let x = if self.axis == VirtualJoystickAxis::Both
            || self.axis == VirtualJoystickAxis::Horizontal
        {
            if self.delta.x > dead_zone {
                1.
            } else if self.delta.x < -dead_zone {
                -1.
            } else {
                0.
            }
        } else {
            0.
        };
        let y = if self.axis == VirtualJoystickAxis::Both
            || self.axis == VirtualJoystickAxis::Vertical
        {
            if self.delta.y > dead_zone {
                1.
            } else if self.delta.y < -dead_zone {
                -1.
            } else {
                0.
            }
        } else {
            0.
        };

        Vec2::new(x, y)
    }
}
