use std::{hash::Hash, marker::PhantomData};

use bevy::{ecs::schedule::ScheduleLabel, prelude::*, reflect::TypePath};

mod bundles;
mod components;
mod systems;
mod utils;

pub use bundles::VirtualJoystickBundle;
pub use components::{
    JoystickDeadZone, JoystickDynamic, JoystickFixed, JoystickFloating, JoystickHorizontalOnly,
    JoystickInvisible, JoystickVerticalOnly, VirtualJoystickNode, VirtualJoystickUIBackground,
    VirtualJoystickUIKnob,
};
use systems::{
    update_dead_zone, update_dynamic, update_fire_events, update_fixed, update_floating, update_horizontal_only, update_input, update_joystick_visible, update_ui, update_vertical_only
};
pub use utils::create_joystick;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UpdateKnobDelta;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstrainKnobDelta;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FireEvents;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UpdateUI;

#[derive(Default)]
pub struct VirtualJoystickPlugin<S> {
    _marker: PhantomData<S>,
}

#[derive(Event)]
pub enum InputEvent {
    StartDrag { id: u64, pos: Vec2, is_mouse: bool },
    Dragging { id: u64, pos: Vec2, is_mouse: bool },
    EndDrag { id: u64, pos: Vec2, is_mouse: bool },
}

pub trait VirtualJoystickID:
    Hash + Sync + Send + Clone + Default + Reflect + TypePath + FromReflect + 'static
{
}

impl<S: Hash + Sync + Send + Clone + Default + Reflect + FromReflect + TypePath + 'static>
    VirtualJoystickID for S
{
}

impl<S: VirtualJoystickID> Plugin for VirtualJoystickPlugin<S> {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<VirtualJoystickNode<S>>()
            .register_type::<VirtualJoystickEventType>()
            .add_event::<VirtualJoystickEvent<S>>()
            .add_event::<InputEvent>()
            .add_systems(PreUpdate, update_input::<S>)
            .add_systems(
                UpdateKnobDelta,
                (update_fixed::<S>, update_floating::<S>, update_dynamic::<S>),
            )
            .add_systems(
                ConstrainKnobDelta,
                (
                    update_dead_zone::<S>,
                    update_horizontal_only::<S>,
                    update_vertical_only::<S>,
                ),
            )
            .add_systems(FireEvents, update_fire_events::<S>)
            .add_systems(UpdateUI, (update_joystick_visible::<S>, update_ui::<S>))
            .add_systems(Update, |world: &mut World| {
                world.run_schedule(UpdateKnobDelta);
                world.run_schedule(ConstrainKnobDelta);
                world.run_schedule(FireEvents);
                world.run_schedule(UpdateUI);
            });
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum VirtualJoystickEventType {
    Press,
    Drag,
    Up,
}

#[derive(Event, Debug)]
pub struct VirtualJoystickEvent<S: VirtualJoystickID> {
    id: S,
    event: VirtualJoystickEventType,
    value: Vec2,
    delta: Vec2,
}

impl<S: VirtualJoystickID> VirtualJoystickEvent<S> {
    /// Get ID of joystick throw event
    pub fn id(&self) -> S {
        self.id.clone()
    }
    /// Raw position of point (Mouse or Touch)
    pub fn value(&self) -> Vec2 {
        self.value
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
        Vec2::new(
            if self.delta.x < -dead_zone {
                -1.0
            } else if self.delta.x > dead_zone {
                1.0
            } else {
                0.0
            },
            if self.delta.y < -dead_zone {
                -1.0
            } else if self.delta.y > dead_zone {
                1.0
            } else {
                0.0
            },
        )
    }
}
