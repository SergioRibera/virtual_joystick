use bevy::prelude::*;

#[cfg(feature = "inspect")]
use bevy_inspector_egui::prelude::*;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "inspect", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "inspect", reflect(Resource, InspectorOptions))]
pub enum VirtualJoystickAxis {
    #[default]
    Both,
    Horizontal,
    Vertical,
}

impl VirtualJoystickAxis {
    pub fn handle(&self, pos: Vec2) -> Vec2 {
        match self {
            VirtualJoystickAxis::Both => pos,
            VirtualJoystickAxis::Horizontal => Vec2::new(pos.x, 0.),
            VirtualJoystickAxis::Vertical => Vec2::new(0., pos.y),
        }
    }
}

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "inspect", derive(Reflect, InspectorOptions))]
#[cfg_attr(feature = "inspect", reflect(Resource, InspectorOptions))]
pub enum VirtualJoystickType {
    /// Static position
    Fixed,
    #[default]
    /// Spawn at point click
    Floating,
    /// Follow point on drag
    Dynamic,
}

impl VirtualJoystickType {
    pub fn handle(&self) {
        match self {
            VirtualJoystickType::Fixed => todo!(),
            VirtualJoystickType::Floating => todo!(),
            VirtualJoystickType::Dynamic => todo!(),
        }
    }
}
