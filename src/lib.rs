use std::{hash::Hash, marker::PhantomData};

use bevy::{prelude::*, reflect::GetTypeRegistration, reflect::TypePath};

mod action;
mod behavior;
mod bundles;
mod components;
mod systems;
mod utils;

pub use action::{NoAction, VirtualJoystickAction};
pub use behavior::{
    JoystickDeadZone, JoystickDynamic, JoystickFixed, JoystickFloating, JoystickHorizontalOnly,
    JoystickInvisible, JoystickVerticalOnly, VirtualJoystickBehavior,
};
pub use bundles::VirtualJoystickBundle;
pub use components::{
    VirtualJoystickInteractionArea, VirtualJoystickNode, VirtualJoystickState,
    VirtualJoystickUIBackground, VirtualJoystickUIKnob,
};
use systems::{
    update_action, update_behavior, update_behavior_constraints, update_behavior_knob_delta,
    update_input, update_missing_state, update_send_messages, update_ui,
};
pub use utils::create_joystick;

#[derive(Default)]
pub struct VirtualJoystickPlugin<S> {
    _marker: PhantomData<S>,
}

#[derive(Message)]
pub enum InputMessage {
    StartDrag { id: u64, pos: Vec2, is_mouse: bool },
    Dragging { id: u64, pos: Vec2, is_mouse: bool },
    EndDrag { id: u64, pos: Vec2, is_mouse: bool },
}

pub trait VirtualJoystickID:
    Hash + Sync + Send + Clone + std::fmt::Debug + Default + Reflect + TypePath + FromReflect + 'static
{
}

impl<
        S: Hash
            + Sync
            + Send
            + Clone
            + std::fmt::Debug
            + Default
            + Reflect
            + FromReflect
            + TypePath
            + 'static,
    > VirtualJoystickID for S
{
}

impl<S: VirtualJoystickID + GetTypeRegistration + bevy::reflect::Typed> Plugin
    for VirtualJoystickPlugin<S>
{
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_type::<VirtualJoystickNode<S>>()
            .register_type::<VirtualJoystickMessageType>()
            .add_message::<VirtualJoystickMessage<S>>()
            .add_message::<InputMessage>()
            .add_systems(
                PreUpdate,
                (
                    update_missing_state::<S>,
                    update_input.after(update_missing_state::<S>),
                ),
            )
            .configure_sets(
                PostUpdate,
                (
                    JoystickSystems::UpdateKnobDelta,
                    JoystickSystems::ConstrainKnobDelta,
                    JoystickSystems::SendMessages,
                    JoystickSystems::UpdateUI,
                )
                    .chain(),
            )
            .add_systems(
                PostUpdate,
                update_behavior_knob_delta::<S>.in_set(JoystickSystems::UpdateKnobDelta),
            )
            .add_systems(
                PostUpdate,
                update_behavior_constraints::<S>.in_set(JoystickSystems::ConstrainKnobDelta),
            )
            .add_systems(
                PostUpdate,
                update_send_messages::<S>.in_set(JoystickSystems::SendMessages),
            )
            .add_systems(
                PostUpdate,
                (update_behavior::<S>, update_action::<S>, update_ui)
                    .in_set(JoystickSystems::UpdateUI),
            );
    }
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub enum JoystickSystems {
    UpdateKnobDelta,
    ConstrainKnobDelta,
    SendMessages,
    UpdateUI,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
#[reflect]
pub enum VirtualJoystickMessageType {
    Press,
    Drag,
    Up,
}

#[derive(Message, Debug)]
pub struct VirtualJoystickMessage<S: VirtualJoystickID> {
    pub id: S,
    pub message_type: VirtualJoystickMessageType,
    pub value: Vec2,
    pub delta: Vec2,
}

impl<S: VirtualJoystickID> VirtualJoystickMessage<S> {
    /// Get ID of `VirtualJoystickMessage`
    pub fn id(&self) -> S {
        self.id.clone()
    }
    /// Raw position of point (Mouse or Touch)
    pub fn value(&self) -> &Vec2 {
        &self.value
    }

    /// Delta value ranging from -1 to 1 in each vector (x and y)
    pub fn axis(&self) -> &Vec2 {
        &self.delta
    }

    /// Return the Type of `VirtualJoystickMessage`
    pub fn get_type(&self) -> VirtualJoystickMessageType {
        self.message_type
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
