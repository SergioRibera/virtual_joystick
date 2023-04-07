use bevy::{
    prelude::*,
    render::RenderApp,
    ui::{RenderUiSystem, UiSystem},
};
#[cfg(feature = "inspect")]
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

mod behaviour;
mod input;
mod joystick;

pub use behaviour::{VirtualJoystickAxis, VirtualJoystickType};
use input::{run_if_pc, update_joystick, update_joystick_by_mouse};
pub use joystick::{
    TintColor, VirtualJoystickBundle, VirtualJoystickInteractionArea, VirtualJoystickNode,
};

use joystick::{extract_joystick_node, VirtualJoystickKnob};

#[derive(Default)]
pub struct VirtualJoystickPlugin;

impl Plugin for VirtualJoystickPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(feature = "inspect")]
        {
            app.add_plugin(ResourceInspectorPlugin::<VirtualJoystickAxis>::default())
                .add_plugin(ResourceInspectorPlugin::<VirtualJoystickType>::default());
        }

        app.register_type::<TintColor>()
            .register_type::<VirtualJoystickInteractionArea>()
            .register_type::<VirtualJoystickNode>()
            .register_type::<VirtualJoystickKnob>()
            .register_type::<VirtualJoystickAxis>()
            .register_type::<VirtualJoystickType>()
            .init_resource::<VirtualJoystickAxis>()
            .init_resource::<VirtualJoystickType>()
            .add_event::<VirtualJoystickEvent>()
            .add_systems((
                update_joystick
                    .run_if(not(run_if_pc))
                    .in_base_set(CoreSet::PreUpdate),
                update_joystick_by_mouse
                    .run_if(run_if_pc)
                    .in_base_set(CoreSet::PreUpdate),
                joystick_image_node_system
                    .before(UiSystem::Flex)
                    .in_base_set(CoreSet::PostUpdate),
            ));

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };
        render_app.add_system(
            extract_joystick_node
                .after(RenderUiSystem::ExtractNode)
                .in_schedule(ExtractSchedule),
        );
    }
}

fn joystick_image_node_system(
    interaction_area: Query<(&Node, With<VirtualJoystickInteractionArea>)>,
    mut joystick: Query<(
        &Transform,
        &VirtualJoystickNode,
        &mut VirtualJoystickKnob,
    )>,
) {
    for (j_pos, data, mut knob) in joystick.iter_mut() {
        let j_pos = j_pos.translation.truncate();
        let Ok((node, _)) = interaction_area.get_single() else {
            return;
        };
        let interaction_area = Rect::from_center_size(j_pos, node.size());
        knob.dead_zone = data.dead_zone;
        knob.interactable_zone_rect = interaction_area;
    }
}

pub struct VirtualJoystickEvent {
    value: Vec2,
    delta: Vec2,
    axis: VirtualJoystickAxis,
}

impl VirtualJoystickEvent {
    /// Raw position of point (Mouse or Touch)
    pub fn value(&self) -> Vec2 {
        self.value
    }

    /// Axis of Joystick see [crate::VirtualJoystickAxis]
    pub fn direction(&self) -> VirtualJoystickAxis {
        self.axis.clone()
    }

    /// Delta value ranging from 0 to 1 in each vector (x and y)
    pub fn axis(&self) -> Vec2 {
        self.delta
    }

    /// Delta value snaped
    /// warn: Still working, not working properly
    #[warn(incomplete_features)]
    pub fn snap_value(&self) -> Vec2 {
        let x = if self.axis == VirtualJoystickAxis::Both
            || self.axis == VirtualJoystickAxis::Horizontal
        {
            if self.delta.x > 0. {
                1.
            } else if self.delta.x < 0. {
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
            if self.delta.y > 0. {
                1.
            } else if self.delta.y < 0. {
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
