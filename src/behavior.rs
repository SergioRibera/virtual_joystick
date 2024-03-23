use std::sync::Arc;

use bevy::{
    ecs::{all_tuples, entity::Entity, world::World},
    hierarchy::Children,
    math::{Rect, Vec2},
    reflect::Reflect,
    render::view::Visibility,
    transform::components::GlobalTransform,
    ui::Node,
};

use crate::{components::VirtualJoystickState, VirtualJoystickUIBackground};

pub trait VirtualJoystickBehavior: Send + Sync + 'static {
    fn update_at_delta_stage(&self, _world: &mut World, _entity: Entity) {}
    fn update_at_constraint_stage(&self, _world: &mut World, _entity: Entity) {}
    fn update(&self, _world: &mut World, _entity: Entity) {}
}

impl<A: VirtualJoystickBehavior + Clone> VirtualJoystickBehavior for Arc<A> {
    fn update_at_delta_stage(&self, world: &mut World, entity: Entity) {
        (**self).update_at_delta_stage(world, entity);
    }
    fn update_at_constraint_stage(&self, world: &mut World, entity: Entity) {
        (**self).update_at_constraint_stage(world, entity);
    }
    fn update(&self, world: &mut World, entity: Entity) {
        (**self).update(world, entity);
    }
}

macro_rules! impl_behavior_sets {
    ($($set: ident),*) => {
        impl<$($set: VirtualJoystickBehavior),*> VirtualJoystickBehavior for ($($set,)*)
        {
            #[allow(non_snake_case)]
            fn update_at_delta_stage(&self, world: &mut World, entity: Entity) {
                let ($($set,)*) = self;
                $($set.update_at_delta_stage(world, entity);)*
            }
            #[allow(non_snake_case)]
            fn update_at_constraint_stage(&self, world: &mut World, entity: Entity) {
                let ($($set,)*) = self;
                $($set.update_at_constraint_stage(world, entity);)*
            }
            #[allow(non_snake_case)]
            fn update(&self, world: &mut World, entity: Entity) {
                let ($($set,)*) = self;
                $($set.update(world, entity);)*
            }
        }
    }
}

all_tuples!(impl_behavior_sets, 1, 20, S);

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct JoystickDeadZone(pub f32);

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct JoystickHorizontalOnly;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct JoystickVerticalOnly;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct JoystickInvisible;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct JoystickFixed;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct JoystickFloating;

#[derive(Clone, Copy, Debug, Default, Reflect)]
pub struct JoystickDynamic;

impl VirtualJoystickBehavior for JoystickDeadZone {
    fn update_at_constraint_stage(&self, world: &mut World, entity: Entity) {
        let Some(mut joystick_state) = world.get_mut::<VirtualJoystickState>(entity) else {
            return;
        };
        let dead_zone = self.0;
        if joystick_state.delta.x.abs() < dead_zone {
            joystick_state.delta.x = 0.0;
        }
        if joystick_state.delta.y.abs() < dead_zone {
            joystick_state.delta.y = 0.0;
        }
    }
}

impl VirtualJoystickBehavior for JoystickHorizontalOnly {
    fn update_at_constraint_stage(&self, world: &mut World, entity: Entity) {
        let Some(mut joystick_state) = world.get_mut::<VirtualJoystickState>(entity) else {
            return;
        };
        joystick_state.delta.y = 0.0;
    }
}

impl VirtualJoystickBehavior for JoystickVerticalOnly {
    fn update_at_constraint_stage(&self, world: &mut World, entity: Entity) {
        let Some(mut joystick_state) = world.get_mut::<VirtualJoystickState>(entity) else {
            return;
        };
        joystick_state.delta.x = 0.0;
    }
}

impl VirtualJoystickBehavior for JoystickInvisible {
    fn update(&self, world: &mut World, entity: Entity) {
        let joystick_state = world.get::<VirtualJoystickState>(entity).map(|x| x.clone());
        let Some(joystick_state) = joystick_state else {
            return;
        };
        let Some(mut joystick_visibility) = world.get_mut::<Visibility>(entity) else {
            return;
        };
        if joystick_state.just_released
            || *joystick_visibility != Visibility::Hidden && joystick_state.touch_state.is_none()
        {
            *joystick_visibility = Visibility::Hidden;
        }
        if let Some(touch_state) = &joystick_state.touch_state {
            if touch_state.just_pressed {
                *joystick_visibility = Visibility::Inherited;
            }
        }
    }
}

impl VirtualJoystickBehavior for JoystickFixed {
    fn update_at_delta_stage(&self, world: &mut World, entity: Entity) {
        let mut children_entities: Vec<Entity> = Vec::new();
        {
            let Some(children) = world.get::<Children>(entity) else {
                return;
            };
            for &child in children.iter() {
                children_entities.push(child);
            }
        }
        let mut joystick_base_rect: Option<Rect> = None;
        for &child in &children_entities {
            if world.get::<VirtualJoystickUIBackground>(child).is_none() {
                continue;
            }
            let Some(joystick_base_node) = world.get::<Node>(child) else {
                continue;
            };
            let Some(joystick_base_global_transform) = world.get::<GlobalTransform>(child) else {
                continue;
            };
            joystick_base_rect =
                Some(joystick_base_node.logical_rect(joystick_base_global_transform));
            break;
        }
        let Some(joystick_base_rect) = joystick_base_rect else {
            return;
        };
        let Some(mut joystick_state) = world.get_mut::<VirtualJoystickState>(entity) else {
            return;
        };
        joystick_state.base_offset = Vec2::ZERO;
        let new_delta: Vec2;
        if let Some(touch_state) = &joystick_state.touch_state {
            let mut new_delta2 = ((touch_state.current - touch_state.start)
                / joystick_base_rect.half_size())
            .clamp(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
            new_delta2.y = -new_delta2.y;
            new_delta = new_delta2;
        } else {
            new_delta = Vec2::ZERO;
        }
        joystick_state.delta = new_delta;
    }
}

impl VirtualJoystickBehavior for JoystickFloating {
    fn update_at_delta_stage(&self, world: &mut World, entity: Entity) {
        let mut children_entities: Vec<Entity> = Vec::new();
        {
            let Some(children) = world.get::<Children>(entity) else {
                return;
            };
            for &child in children.iter() {
                children_entities.push(child);
            }
        }
        let mut joystick_base_rect: Option<Rect> = None;
        for &child in &children_entities {
            if world.get::<VirtualJoystickUIBackground>(child).is_none() {
                continue;
            }
            let Some(joystick_base_node) = world.get::<Node>(child) else {
                continue;
            };
            let Some(joystick_base_global_transform) = world.get::<GlobalTransform>(child) else {
                continue;
            };
            joystick_base_rect =
                Some(joystick_base_node.logical_rect(joystick_base_global_transform));
            break;
        }
        let Some(joystick_base_rect) = joystick_base_rect else {
            return;
        };
        let Some(mut joystick_state) = world.get_mut::<VirtualJoystickState>(entity) else {
            return;
        };
        let base_offset: Vec2;
        let mut assign_base_offset = false;
        if let Some(touch_state) = &joystick_state.touch_state {
            if touch_state.just_pressed {
                base_offset = touch_state.start - joystick_base_rect.center();
                assign_base_offset = true;
            } else {
                base_offset = joystick_state.base_offset;
            }
        } else if joystick_state.just_released {
            base_offset = Vec2::ZERO;
            assign_base_offset = true;
        } else {
            base_offset = joystick_state.base_offset;
        }
        if assign_base_offset {
            joystick_state.base_offset = base_offset;
        }
        let new_delta: Vec2;
        if let Some(touch_state) = &joystick_state.touch_state {
            let mut new_delta2 = ((touch_state.current - touch_state.start)
                / joystick_base_rect.half_size())
            .clamp(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
            new_delta2.y = -new_delta2.y;
            new_delta = new_delta2;
        } else {
            new_delta = Vec2::ZERO;
        }
        joystick_state.delta = new_delta;
    }
}

impl VirtualJoystickBehavior for JoystickDynamic {
    fn update_at_delta_stage(&self, world: &mut World, entity: Entity) {
        let joystick_rect: Rect;
        {
            let Some(joystick_node) = world.get::<Node>(entity) else {
                return;
            };
            let Some(joystick_global_transform) = world.get::<GlobalTransform>(entity) else {
                return;
            };
            joystick_rect = joystick_node.logical_rect(joystick_global_transform);
        }
        let mut children_entities: Vec<Entity> = Vec::new();
        {
            let Some(children) = world.get::<Children>(entity) else {
                return;
            };
            for &child in children.iter() {
                children_entities.push(child);
            }
        }
        let mut joystick_base_rect: Option<Rect> = None;
        for &child in &children_entities {
            if world.get::<VirtualJoystickUIBackground>(child).is_none() {
                continue;
            }
            let Some(joystick_base_node) = world.get::<Node>(child) else {
                continue;
            };
            let Some(joystick_base_global_transform) = world.get::<GlobalTransform>(child) else {
                continue;
            };
            joystick_base_rect =
                Some(joystick_base_node.logical_rect(joystick_base_global_transform));
            break;
        }
        let Some(joystick_base_rect) = joystick_base_rect else {
            return;
        };
        let Some(mut joystick_state) = world.get_mut::<VirtualJoystickState>(entity) else {
            return;
        };
        let joystick_base_rect_center = joystick_base_rect.center();
        let joystick_base_rect_half_size = joystick_base_rect.half_size();
        let base_offset: Vec2;
        let mut assign_base_offset = false;
        if let Some(touch_state) = &joystick_state.touch_state {
            if touch_state.just_pressed {
                base_offset = touch_state.start - joystick_base_rect_center;
                assign_base_offset = true;
            } else {
                base_offset = joystick_state.base_offset;
            }
        } else if joystick_state.just_released {
            base_offset = Vec2::ZERO;
            assign_base_offset = true;
        } else {
            base_offset = joystick_state.base_offset;
        }
        if assign_base_offset {
            joystick_state.base_offset = base_offset;
        }
        let new_delta: Vec2;
        let mut new_base_offset: Option<Vec2> = None;
        if let Some(touch_state) = &joystick_state.touch_state {
            let mut offset = touch_state.current
                - (joystick_rect.min + base_offset + joystick_base_rect.half_size());
            if offset.length_squared()
                > joystick_base_rect_half_size.x * joystick_base_rect_half_size.x
            {
                let adjustment =
                    offset - offset * (joystick_base_rect_half_size.x / offset.length());
                offset += adjustment;
                new_base_offset = Some(base_offset + adjustment);
            }
            let mut new_delta2 = (offset / joystick_base_rect_half_size)
                .clamp(Vec2::new(-1.0, -1.0), Vec2::new(1.0, 1.0));
            new_delta2.y = -new_delta2.y;
            new_delta = new_delta2;
        } else {
            new_delta = Vec2::ZERO;
        }
        joystick_state.delta = new_delta;
        if let Some(base_offset) = new_base_offset {
            joystick_state.base_offset = base_offset;
        }
    }
}
