use bevy::{
    prelude::*,
    render::Extract,
    ui::{ExtractedUiNodes, RelativeCursorPosition},
};

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

#[allow(clippy::type_complexity)]
pub fn extract_joystick_node<S: VirtualJoystickID>(
    mut extracted_uinodes: ResMut<ExtractedUiNodes>,
    knob_ui_query: Extract<Query<(Entity, &Parent), With<VirtualJoystickUIKnob>>>,
    bg_ui_query: Extract<Query<(Entity, &Parent), With<VirtualJoystickUIBackground>>>,
    uinode_query: Extract<
        Query<(
            &Node,
            &GlobalTransform,
            &VirtualJoystickNode<S>,
            &Visibility,
            &InheritedVisibility,
            &ViewVisibility,
            &VirtualJoystickData,
        )>,
    >,
) {
    for (entity, parent) in &knob_ui_query {
        if let Ok((
            uinode,
            global_transform,
            joystick_node,
            visibility,
            inherited_visibility,
            view_visibility,
            data,
        )) = uinode_query.get(**parent)
        {
            if visibility == Visibility::Hidden
                || !inherited_visibility.get()
                || !view_visibility.get()
                || uinode.size().x == 0.
                || uinode.size().y == 0.
                || data.id_drag.is_none() && joystick_node.behaviour == VirtualJoystickType::Dynamic
            {
                continue;
            }
            let base_pos = get_base_pos(uinode, joystick_node.behaviour, data, global_transform);
            let radius = uinode.size().x / 2.;
            // ui is y down, so we flip
            let pos = -data.delta * radius;
            let knob_pos = base_pos + joystick_node.axis.handle_vec3(pos.extend(0.));

            extracted_uinodes
                .uinodes
                .entry(entity)
                .and_modify(|node| {
                    node.transform = Mat4::from_translation(knob_pos);
                });
        }
    }

    for (entity, parent) in &bg_ui_query {
        if let Ok((
            uinode,
            global_transform,
            joystick_node,
            visibility,
            inherited_visibility,
            view_visibility,
            data,
        )) = uinode_query.get(**parent)
        {
            if visibility == Visibility::Hidden
                || !inherited_visibility.get()
                || !view_visibility.get()
                || uinode.size().x == 0.
                || uinode.size().y == 0.
                || data.id_drag.is_none() && joystick_node.behaviour == VirtualJoystickType::Dynamic
            {
                continue;
            }
            let pos = get_base_pos(uinode, joystick_node.behaviour, data, global_transform);
            extracted_uinodes
                .uinodes
                .entry(entity)
                .and_modify(|node| {
                    node.transform = Mat4::from_translation(pos);
                });
        }
    }
}

fn get_base_pos(
    uinode: &Node,
    behaviour: VirtualJoystickType,
    joystick: &VirtualJoystickData,
    global_transform: &GlobalTransform,
) -> Vec3 {
    let container_rect = Rect {
        max: uinode.size(),
        ..default()
    };

    let border_pos = match behaviour {
        VirtualJoystickType::Fixed => global_transform
            .compute_matrix()
            .transform_point3((container_rect.center() - (uinode.size() / 2.)).extend(0.)),
        VirtualJoystickType::Floating => {
            if joystick.id_drag.is_none() {
                global_transform
                    .compute_matrix()
                    .transform_point3((container_rect.center() - (uinode.size() / 2.)).extend(0.))
            } else {
                joystick.start_pos.extend(0.)
            }
        }
        VirtualJoystickType::Dynamic => joystick.base_pos.extend(0.),
    };

    border_pos
}
