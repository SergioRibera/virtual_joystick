use bevy::{
    ecs::bundle::Bundle,
    prelude::default,
    render::view::{InheritedVisibility, ViewVisibility, Visibility},
    transform::components::{GlobalTransform, Transform},
    ui::{Node, Style, ZIndex},
};

use crate::{VirtualJoystickID, VirtualJoystickNode};

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
