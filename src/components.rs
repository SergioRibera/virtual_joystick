use bevy::{
    ecs::{component::Component, reflect::ReflectComponent},
    prelude::Vec2,
    reflect::{std_traits::ReflectDefault, Reflect},
};
#[cfg(feature = "inspect")]
use bevy_inspector_egui::prelude::ReflectInspectorOptions;
#[cfg(feature = "inspect")]
use bevy_inspector_egui::InspectorOptions;

use crate::VirtualJoystickID;

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

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct VirtualJoystickNode<S: VirtualJoystickID> {
    pub id: S,
    pub touch_state: Option<TouchState>,
    pub just_released: bool,
    pub base_offset: Vec2,
    pub delta: Vec2,
}

impl<S: VirtualJoystickID> VirtualJoystickNode<S> {
    pub fn with_id(mut self, id: S) -> Self {
        self.id = id;
        self
    }
}

#[derive(Clone, Debug, Default, Reflect)]
#[reflect(Default)]
pub struct TouchState {
    pub id: u64,
    pub is_mouse: bool,
    pub start: Vec2,
    pub current: Vec2,
    pub just_pressed: bool,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickDeadZone(pub f32);

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickHorizontalOnly;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickVerticalOnly;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickInvisible;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickFixed;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickFloating;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct JoystickDynamic;
