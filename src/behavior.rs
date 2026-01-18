use std::sync::Arc;

use bevy::{
    ecs::{entity::Entity, world::World},
    math::{FloatPow, Rect, Vec2},
    prelude::{Children, Visibility},
    reflect::Reflect,
    ui::{ComputedNode, UiGlobalTransform},
};
use variadics_please::all_tuples;

use crate::{
    components::{TouchState, VirtualJoystickState},
    VirtualJoystickUIBackground,
};

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
        let joystick_state = world.get::<VirtualJoystickState>(entity).cloned();
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
        let Some(joystick_base_rect) = joystick_base_rect(&*world, entity) else {
            return;
        };
        let Some(mut joystick_state) = world.get_mut::<VirtualJoystickState>(entity) else {
            return;
        };

        joystick_state.base_offset = Vec2::ZERO;

        // Return if `touch_state` is `None` and set delta to `ZERO`.
        let Some(touch_state) = &joystick_state.touch_state else {
            joystick_state.delta = Vec2::ZERO;
            return;
        };

        // Set `joystick_state.delta`.
        let offset = touch_state.current - joystick_base_rect.center();
        joystick_state.delta = joystick_delta(joystick_base_rect, offset, false);
    }
}

impl VirtualJoystickBehavior for JoystickDynamic {
    fn update_at_delta_stage(&self, world: &mut World, entity: Entity) {
        let Some(joystick_rect) = joystick_rect(world, entity) else {
            return;
        };
        let Some(joystick_base_rect) = joystick_base_rect(&*world, entity) else {
            return;
        };
        let Some(mut joystick_state) = world.get_mut::<VirtualJoystickState>(entity) else {
            return;
        };

        // Return if `touch_state` is `None` and set delta to `ZERO`.
        let Some(touch_state) = update_base_offset(&mut joystick_state, joystick_base_rect) else {
            joystick_state.delta = Vec2::ZERO;
            return;
        };

        // Set `joystick_state.delta`.
        let offset = touch_state.current
            - (joystick_rect.min + joystick_state.base_offset + joystick_base_rect.half_size());
        joystick_state.delta = joystick_delta(joystick_base_rect, offset, false);

        // Calculate appropriate `delta` and add to `joystick_state.base_offset` if appropriate.
        if let Some(delta) = base_offset_delta(joystick_base_rect, offset) {
            joystick_state.base_offset += delta;
        }
    }
}

impl VirtualJoystickBehavior for JoystickFloating {
    fn update_at_delta_stage(&self, world: &mut World, entity: Entity) {
        let Some(joystick_base_rect) = joystick_base_rect_scaled(&*world, entity) else {
            return;
        };
        let Some(mut joystick_state) = world.get_mut::<VirtualJoystickState>(entity) else {
            return;
        };

        // Return if `touch_state` is `None` or `touch_state.just_pressed` and set delta to `ZERO`.
        let Some(touch_state) = update_base_offset(&mut joystick_state, joystick_base_rect) else {
            joystick_state.delta = Vec2::ZERO;
            return;
        };
        if touch_state.just_pressed {
            joystick_state.delta = Vec2::ZERO;
            return;
        }

        // Set `joystick_state.delta`.
        let offset = touch_state.current - joystick_base_rect.center();
        joystick_state.delta = joystick_delta(joystick_base_rect, offset, true);
    }
}

/// The [`Rect`] of the joystick returned as an [`Option`].
fn joystick_rect(world: &World, entity: Entity) -> Option<Rect> {
    let node = world.get::<ComputedNode>(entity)?;
    let transform = world.get::<UiGlobalTransform>(entity)?;

    Some(Rect::from_center_size(transform.translation, node.size()))
}

/// The [`Rect`] of the joystick base returned as an [`Option`].
///
/// Compared to [`joystick_rect`], this gives the [`Rect`] of the associated [`VirtualJoystickUIBackground`], which
/// is part of a child [`Entity`] of the joystick.
fn joystick_base_rect(world: &World, entity: Entity) -> Option<Rect> {
    let (translation, node) = joystick_base_rect_params(world, entity)?;

    Some(Rect::from_center_size(translation, node.size()))
}

/// The scaled [`Rect`] of the joystick base returned as an [`Option`].
///
/// Compared to [`joystick_base_rect`], this just scales the [`Rect`] by [`ComputedNode::inverse_scale_factor`]
/// which is needed to get the logical coordinates.
///
/// Compared to [`joystick_rect`], this gives the [`Rect`] of the associated [`VirtualJoystickUIBackground`], which
/// is part of a child [`Entity`] of the joystick.
fn joystick_base_rect_scaled(world: &World, entity: Entity) -> Option<Rect> {
    let (translation, node) = joystick_base_rect_params(world, entity)?;
    let factor = node.inverse_scale_factor;

    Some(Rect::from_center_size(
        translation * factor,
        node.size() * factor,
    ))
}

/// Parameters for the base [`Rect`].
fn joystick_base_rect_params(world: &World, entity: Entity) -> Option<(Vec2, &ComputedNode)> {
    let children = world.get::<Children>(entity)?;
    let base = children
        .iter()
        .find(|entity| world.get::<VirtualJoystickUIBackground>(**entity).is_some())?;
    let node = world.get::<ComputedNode>(*base)?;
    let transform = world.get::<UiGlobalTransform>(*base)?;

    Some((transform.translation, node))
}

/// Update [`VirtualJoystickState::base_offset`] and return the associated [`TouchState`] as an [`Option`].
fn update_base_offset(state: &mut VirtualJoystickState, rect: Rect) -> Option<&TouchState> {
    // Return None if `state.touch_state` is `None` and set `state.base_offset` to ZERO if joystick was just released.
    let Some(touch_state) = &state.touch_state else {
        if state.just_released {
            state.base_offset = Vec2::ZERO;
        }
        return None;
    };

    // Center `state.base_offset` from starting point if joystick was just pressed and return `touch_state`.
    if touch_state.just_pressed {
        state.base_offset = touch_state.start - rect.center();
    }
    Some(touch_state)
}

/// The normalized delta for [`VirtualJoystickState::delta`] calculated from an offset
fn joystick_delta(rect: Rect, offset: Vec2, is_floating: bool) -> Vec2 {
    let half_size = rect.half_size();
    let distance_squared = offset.length_squared();

    // Clamp offset to circular bounds with radius `half_size.x`.
    let offset = if distance_squared > half_size.x.squared() {
        offset * half_size.x / distance_squared.sqrt()
    } else {
        offset
    };
    // Normalize based on `is_floating`.
    // If `is_floating` use circular bounds with radius `half_size.x`,
    // otherwise use bounds of `rect`.
    let normalizer = if is_floating {
        Vec2::splat(half_size.x)
    } else {
        half_size
    };

    // Return normalized offset, now also clamped to be between `-1.` and `1.`.
    // NOTE: We are inverting y to align with user intent because `offset` is reversed on the y axis.
    let Vec2 { x, y } = (offset / normalizer).clamp(Vec2::splat(-1.), Vec2::splat(1.));
    Vec2::new(x, -y)
}

/// The normalized delta for [`VirtualJoystickState::base_offset`] calculated from an offset
fn base_offset_delta(rect: Rect, offset: Vec2) -> Option<Vec2> {
    let half_size_x = rect.half_size().x;
    let distance_squared = offset.length_squared();
    // Return `None` if circular bounds with radius `half_size.x` have not been reached.
    if distance_squared <= half_size_x.squared() {
        return None;
    }

    // Return accurate `offset` for clamping to circular bounds with radius `half_size.x` if adding
    // it to `VirtualJoystickState::base_offset`
    let distance = distance_squared.sqrt();
    let offset = offset * half_size_x / distance;
    Some(offset * (1. - half_size_x / distance))
}
