use std::marker::PhantomData;

use bevy::ecs::{all_tuples, entity::Entity, world::World};

use crate::VirtualJoystickState;

pub trait VirtualJoystickAction<I>: Send + Sync + 'static {
    fn on_start_drag(
        &self,
        _id: I,
        _data: VirtualJoystickState,
        _world: &mut World,
        _entity: Entity,
    ) {
    }
    fn on_drag(&self, _id: I, _data: VirtualJoystickState, _world: &mut World, _entity: Entity) {}
    fn on_end_drag(
        &self,
        _id: I,
        _data: VirtualJoystickState,
        _world: &mut World,
        _entity: Entity,
    ) {
    }
}

#[derive(Default)]
pub struct NoAction;

impl<I> VirtualJoystickAction<I> for NoAction {}
