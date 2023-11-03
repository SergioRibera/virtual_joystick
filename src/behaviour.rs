use bevy::prelude::*;

#[cfg(feature = "inspect")]
use bevy_inspector_egui::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
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

    pub fn handle_xy(&self, x: f32, y: f32) -> Vec2 {
        match self {
            VirtualJoystickAxis::Both => Vec2::new(x, y),
            VirtualJoystickAxis::Horizontal => Vec2::new(x, 0.),
            VirtualJoystickAxis::Vertical => Vec2::new(0., y),
        }
    }

    pub fn handle_vec3(&self, pos: Vec3) -> Vec3 {
        self.handle_xy(pos.x, pos.y).extend(pos.z)
    }
}

#[derive(Reflect, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
pub enum VirtualJoystickType {
    /// Static position
    Fixed,
    #[default]
    /// Spawn at point click
    Floating,
    /// Follow point on drag
    Dynamic,
}
