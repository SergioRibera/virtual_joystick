use std::sync::Arc;

use bevy::{
    ecs::{component::Component, reflect::ReflectComponent},
    prelude::Vec2,
    reflect::{Reflect, std_traits::ReflectDefault},
};
#[cfg(feature = "inspect")]
use bevy_inspector_egui::InspectorOptions;
#[cfg(feature = "inspect")]
use bevy_inspector_egui::prelude::ReflectInspectorOptions;

use crate::{
    VirtualJoystickAction, VirtualJoystickBehavior, VirtualJoystickID, action::NoAction,
    behavior::JoystickFloating,
};

#[derive(Component, Copy, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
pub struct VirtualJoystickUIKnob;

#[derive(Component, Copy, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
pub struct VirtualJoystickUIBackground;

#[derive(Component, Copy, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
pub struct VirtualJoystickInteractionArea;

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct VirtualJoystickNode<S: VirtualJoystickID> {
    pub id: S,
    #[reflect(ignore)]
    pub behavior: Arc<dyn VirtualJoystickBehavior>,
    #[reflect(ignore)]
    pub action: Arc<dyn VirtualJoystickAction<S>>,
}

impl<S: VirtualJoystickID> Default for VirtualJoystickNode<S> {
    fn default() -> Self {
        Self {
            id: Default::default(),
            behavior: Arc::new(JoystickFloating),
            action: Arc::new(NoAction),
        }
    }
}

impl<S: VirtualJoystickID> std::fmt::Debug for VirtualJoystickNode<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualJoystickNode")
            .field("id", &self.id)
            .finish()
    }
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct VirtualJoystickState {
    pub touch_state: Option<TouchState>,
    pub just_released: bool,
    pub base_offset: Vec2,
    pub delta: Vec2,
}
impl VirtualJoystickState {
    // Reset input related fields.
    pub fn reset_input(&mut self) {
        self.touch_state = None;
        self.just_released = true;
    }
}

impl<S: VirtualJoystickID> VirtualJoystickNode<S> {
    pub fn with_id(mut self, id: S) -> Self {
        self.id = id;
        self
    }

    pub fn with_behavior(mut self, behavior: impl VirtualJoystickBehavior) -> Self {
        self.behavior = Arc::new(behavior);
        self
    }

    pub fn with_action(mut self, action: impl VirtualJoystickAction<S>) -> Self {
        self.action = Arc::new(action);
        self
    }
}

#[derive(Clone, Debug, Default, Reflect)]
#[reflect(Default)]
pub struct TouchState {
    pub id: Option<u64>,
    pub start: Vec2,
    pub current: Vec2,
    pub just_pressed: bool,
}

impl TouchState {
    /// Set new [`Self::current`].
    pub fn set_new_current(&mut self, new_current: Vec2) {
        if self.current != new_current {
            self.current = new_current;
        }
    }
    /// Initialize new touch state.
    pub fn new(id: Option<u64>, pos: Vec2) -> Self {
        Self {
            id,
            start: pos,
            current: pos,
            just_pressed: true,
        }
    }
}
