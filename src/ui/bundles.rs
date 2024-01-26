use bevy::{prelude::*, ui::RelativeCursorPosition};

#[cfg(feature = "inspect")]
use bevy_inspector_egui::prelude::*;

use crate::{VirtualJoystickAxis, VirtualJoystickID, VirtualJoystickType};

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
#[cfg_attr(feature = "inspect", derive(InspectorOptions))]
#[cfg_attr(feature = "inspect", reflect(InspectorOptions))]
pub struct VirtualJoystickInteractionArea;

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

#[derive(Bundle, Debug, Default)]
pub struct VirtualJoystickBundle<S: VirtualJoystickID> {
    /// Describes the size of the node
    pub(crate) node: Node,
    /// Describes the style including flexbox settings
    pub(crate) style: Style,
    /// The texture atlas image of the node
    pub(crate) joystick: VirtualJoystickNode<S>,
    /// The transform of the node
    pub(crate) transform: Transform,
    /// The global transform of the node
    pub(crate) global_transform: GlobalTransform,
    /// Describes the visibility properties of the node
    pub visibility: Visibility,
    /// Inherited visibility of an entity.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub view_visibility: ViewVisibility,
    /// Indicates the depth at which the node should appear in the UI
    pub(crate) z_index: ZIndex,
    pub(crate) knob_data: VirtualJoystickData,
    pub(crate) cursor_pos: RelativeCursorPosition,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct VirtualJoystickNode<S: VirtualJoystickID> {
    /// Identifier of joystick
    pub id: S,
    /// Zone to ignore movement
    pub dead_zone: f32,
    /// Define Axis for this joystick
    pub axis: VirtualJoystickAxis,
    /// Define the behaviour of joystick
    pub behaviour: VirtualJoystickType,
}

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component, Default)]
pub struct VirtualJoystickData {
    pub id_drag: Option<u64>,
    pub dead_zone: f32,
    pub base_pos: Vec2,
    pub start_pos: Vec2,
    pub current_pos: Vec2,
    pub delta: Vec2,
    pub interactable_zone_rect: Rect,
    /// None means no current interaction<br/>
    /// Some(false) means current interaction is touch<br/>
    /// Some(true) means current interaction is mouse
    pub current_iteraction_is_mouse: Option<bool>,
}

impl<S: VirtualJoystickID> VirtualJoystickBundle<S> {
    pub fn new(joystick: VirtualJoystickNode<S>) -> Self {
        Self {
            joystick,
            ..default()
        }
    }

    pub fn set_node(mut self, node: Node) -> Self {
        self.node = node;
        self
    }

    pub fn set_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn set_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    pub fn set_global_transform(mut self, global_transform: GlobalTransform) -> Self {
        self.global_transform = global_transform;
        self
    }

    pub fn set_z_index(mut self, z_index: ZIndex) -> Self {
        self.z_index = z_index;
        self
    }
}
