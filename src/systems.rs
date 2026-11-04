use std::sync::Arc;

use bevy::{
    ecs::{
        entity::Entity,
        message::MessageWriter,
        query::{QueryState, With},
        system::{Query, Res, Single},
        world::World,
    },
    input::{ButtonInput, mouse::MouseButton, touch::Touches},
    math::{Rect, Vec2},
    prelude::Children,
    ui::{ComputedNode, Node, PositionType, UiGlobalTransform, UiScale, Val},
    window::{PrimaryWindow, Window},
};

use crate::{
    VirtualJoystickID, VirtualJoystickMessage, VirtualJoystickMessageType, VirtualJoystickNode,
    components::{
        PointerState, VirtualJoystickInteractionArea, VirtualJoystickState,
        VirtualJoystickUIBackground, VirtualJoystickUIKnob,
    },
};
use bevy::ecs::query::Without;

/// Current action being performed by the mouse/touch input
enum DragAction {
    Start,
    Move,
    End,
}

/// Add missing [`VirtualJoystickState`]s for [`Entity`]s with [`VirtualJoystickNode`]
pub fn update_missing_state<S: VirtualJoystickID>(
    world: &mut World,
    joysticks: &mut QueryState<Entity, With<VirtualJoystickNode<S>>>,
) {
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    for entity in joysticks {
        if world.get::<VirtualJoystickState>(entity).is_none() {
            world
                .entity_mut(entity)
                .insert(VirtualJoystickState::default());
        }
    }
}

/// Update stored inputs in [`VirtualJoystickState`].
pub fn update_input(
    window: Single<&Window, With<PrimaryWindow>>,
    joystick_query: Query<(
        Entity,
        &ComputedNode,
        &UiGlobalTransform,
        &mut VirtualJoystickState,
    )>,
    children_query: Query<&Children>,
    interaction_area_query: Query<
        (&ComputedNode, &UiGlobalTransform),
        With<VirtualJoystickInteractionArea>,
    >,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    ui_scale: Res<UiScale>,
) {
    for (entity, node, transform, mut state) in joystick_query {
        state.just_released = false;

        if let Some(pointer_state) = &mut state.pointer_state {
            pointer_state.just_pressed = false;

            // Either reset `state` input if it is not pressed or update with current position.
            if let Some(id) = pointer_state.id {
                if let Some(touch) = touches.get_pressed(id) {
                    pointer_state.set_new_current(touch.position());
                } else {
                    state.reset_input();
                }
            } else {
                if !mouse_buttons.pressed(MouseButton::Left) {
                    state.reset_input();
                } else if let Some(current) = window.cursor_position() {
                    pointer_state.set_new_current(current);
                }
            }
        } else {
            // Get rect or fallback to default calculated with `transform` from `joystick_query`.
            let rect = if let Some(children) = children_query.get(entity).into_iter().next()
                && let Some((node, interaction_transform)) = children
                    .iter()
                    .find_map(|&child| interaction_area_query.get(child).ok())
            {
                node_rect(node, interaction_transform.translation, ui_scale.0)
            } else {
                node_rect(node, transform.translation, ui_scale.0)
            };

            // Set `pointer_state` according to if input is in `interaction_rect`.
            state.pointer_state =
                if let Some(touch) = touches.iter().find(|touch| rect.contains(touch.position())) {
                    Some(PointerState::new(Some(touch.id()), touch.position()))
                } else if mouse_buttons.just_pressed(MouseButton::Left)
                    && let Some(mouse_pos) = window.cursor_position()
                    && rect.contains(mouse_pos)
                {
                    Some(PointerState::new(None, mouse_pos))
                } else {
                    None
                };
        }
    }
}

/// Update behavior knob delta by calling [`crate::behavior::VirtualJoystickBehavior::update_at_delta_stage`] for each joystick entity.
pub fn update_behavior_knob_delta<S: VirtualJoystickID>(
    world: &mut World,
    joysticks: &mut QueryState<Entity, With<VirtualJoystickNode<S>>>,
) {
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    for entity in joysticks {
        let behavior = {
            let Some(virtual_joystick_node) = world.get::<VirtualJoystickNode<S>>(entity) else {
                continue;
            };
            Arc::clone(&virtual_joystick_node.behavior)
        };
        behavior.update_at_delta_stage(world, entity);
    }
}

/// Update behavior constraints by calling [`crate::behavior::VirtualJoystickBehavior::update_at_constraint_stage`] for each joystick entity.
pub fn update_behavior_constraints<S: VirtualJoystickID>(
    world: &mut World,
    joysticks: &mut QueryState<Entity, With<VirtualJoystickNode<S>>>,
) {
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    for entity in joysticks {
        let behavior = {
            let Some(virtual_joystick_node) = world.get::<VirtualJoystickNode<S>>(entity) else {
                continue;
            };
            Arc::clone(&virtual_joystick_node.behavior)
        };
        behavior.update_at_constraint_stage(world, entity);
    }
}

/// Update behavior by calling [`crate::behavior::VirtualJoystickBehavior::update`] for each joystick entity.
pub fn update_behavior<S: VirtualJoystickID>(
    world: &mut World,
    joysticks: &mut QueryState<Entity, With<VirtualJoystickNode<S>>>,
) {
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    for entity in joysticks {
        let behavior = {
            let Some(virtual_joystick_node) = world.get::<VirtualJoystickNode<S>>(entity) else {
                continue;
            };
            Arc::clone(&virtual_joystick_node.behavior)
        };
        behavior.update(world, entity);
    }
}

/// Update [`crate::VirtualJoystickAction`] from [`VirtualJoystickState`].
pub fn update_action<S: VirtualJoystickID>(
    world: &mut World,
    joysticks: &mut QueryState<(Entity, &VirtualJoystickNode<S>, &mut VirtualJoystickState)>,
) {
    let joysticks: Vec<_> = joysticks.iter(world).collect();

    // Collect actions to be executed
    let mut actions = Vec::new();
    for (entity, node, state) in joysticks {
        let Some(joystick_state) = world.get::<VirtualJoystickState>(entity) else {
            continue;
        };
        let drag_action = if joystick_state.just_released {
            DragAction::End
        } else if let Some(pointer_state) = &joystick_state.pointer_state {
            if pointer_state.just_pressed {
                DragAction::Start
            } else {
                DragAction::Move
            }
        } else {
            continue;
        };

        actions.push((
            node.id.clone(),
            Arc::clone(&node.action),
            drag_action,
            state.clone(),
            entity,
        ));
    }
    // Execute appropriate actions for `drag_action`s
    for (id, action, drag_action, state, entity) in actions {
        match drag_action {
            DragAction::Start => {
                action.on_start_drag(id, state, world, entity);
            }
            DragAction::Move => {
                action.on_drag(id, state, world, entity);
            }
            DragAction::End => {
                action.on_end_drag(id, state, world, entity);
            }
        }
    }
}

/// Send [VirtualJoystickMessage]s from [`VirtualJoystickState`].
pub fn update_send_messages<S: VirtualJoystickID>(
    joystick_query: Query<(&VirtualJoystickNode<S>, &VirtualJoystickState)>,
    mut writer: MessageWriter<VirtualJoystickMessage<S>>,
) {
    for (joystick, state) in joystick_query {
        let id = joystick.id.clone();
        let delta = state.delta;
        let Some((message_type, value)) = message_type_and_value(state) else {
            continue;
        };

        writer.write(VirtualJoystickMessage {
            id,
            message_type,
            value,
            delta,
        });
    }
}

/// Update visual representation of the joysticks by interpreting [`VirtualJoystickState`].
#[allow(clippy::complexity)]
pub fn update_ui(
    mut joystick_base_query: Query<
        (&mut Node, &ComputedNode, &UiGlobalTransform),
        With<VirtualJoystickUIBackground>,
    >,
    mut joystick_knob_query: Query<
        (&mut Node, &ComputedNode, &UiGlobalTransform),
        (
            With<VirtualJoystickUIKnob>,
            Without<VirtualJoystickUIBackground>,
        ),
    >,
    joystick_query: Query<(&VirtualJoystickState, &Children)>,
) {
    for (joystick_state, children) in joystick_query {
        let Some(base) = children
            .iter()
            .find(|entity| joystick_base_query.contains(**entity))
        else {
            return;
        };
        let (mut base_style, base_node, base_global_transform) =
            joystick_base_query.get_mut(*base).unwrap();

        // Adjust position of base to match `joystick_state.base_offset`
        base_style.position_type = PositionType::Absolute;
        base_style.left = Val::Px(joystick_state.base_offset.x);
        base_style.top = Val::Px(joystick_state.base_offset.y);

        // NOTE: This uses 1. for `ui_scale` to avoid double scaling since
        //       this is later used in a calculation of a `Node` position.
        let base_rect_half_size =
            node_rect(base_node, base_global_transform.translation, 1.).half_size();

        let Some(knob) = children
            .iter()
            .find(|entity| joystick_knob_query.contains(**entity))
        else {
            return;
        };
        let (mut knob_style, knob_node, knob_global_transform) =
            joystick_knob_query.get_mut(*knob).unwrap();
        // NOTE: This uses 1. for `ui_scale` to avoid double scaling since
        //       this is later used in a calculation of a `Node` position.
        let knob_rect_half_size =
            node_rect(knob_node, knob_global_transform.translation, 1.).half_size();

        // Adjust position of knob to match correct axial movement.
        let delta = joystick_state.delta;
        // NOTE: We are inverting y to align with user intent because `offset` is reversed on the y axis.
        let delta = Vec2::new(delta.x, -delta.y);
        let Vec2 { x, y } = joystick_state.base_offset
            + base_rect_half_size
            + knob_rect_half_size
            + base_rect_half_size * (delta - 1.);
        knob_style.position_type = PositionType::Absolute;
        knob_style.left = Val::Px(x);
        knob_style.top = Val::Px(y);
    }
}

/// The [`Rect`] of a [`ComputedNode`].
fn node_rect(node: &ComputedNode, translation: Vec2, ui_scale: f32) -> Rect {
    let factor = node.inverse_scale_factor * ui_scale;
    Rect::from_center_size(translation * factor, node.size() * factor)
}

/// The appropriate [`VirtualJoystickMessageType`] and the appropriate [`VirtualJoystickMessage::value`] from [`VirtualJoystickState`].
fn message_type_and_value(
    state: &VirtualJoystickState,
) -> Option<(VirtualJoystickMessageType, Vec2)> {
    if state.just_released {
        Some((VirtualJoystickMessageType::Up, Vec2::ZERO))
    } else {
        state.pointer_state.as_ref().map(|pointer_state| {
            if pointer_state.just_pressed {
                (VirtualJoystickMessageType::Press, pointer_state.current)
            } else {
                (VirtualJoystickMessageType::Drag, pointer_state.current)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use bevy::{
        ecs::message::{MessageCursor, Messages},
        input::touch::{TouchInput, TouchPhase, touch_screen_input_system},
        prelude::{
            App, ChildOf, Component, IntoScheduleConfigs, MinimalPlugins, PreUpdate, Resource,
        },
    };

    use super::*;
    use crate::{
        JoystickDeadZone, JoystickFixed, JoystickHorizontalOnly, VirtualJoystickAction,
        VirtualJoystickBehavior, VirtualJoystickBundle,
        VirtualJoystickMessageType::{Drag, Press, Up},
        VirtualJoystickPlugin,
    };

    type Cursor = MessageCursor<VirtualJoystickMessage<String>>;

    fn test_app() -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<Touches>()
            .init_resource::<UiScale>()
            .add_message::<TouchInput>()
            .add_plugins(VirtualJoystickPlugin::<String>::default())
            .add_systems(PreUpdate, touch_screen_input_system.before(update_input));
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        (app, window)
    }

    fn ui_geometry(size: f32) -> (Node, ComputedNode, UiGlobalTransform) {
        (
            Node::default(),
            ComputedNode {
                size: Vec2::splat(size),
                ..Default::default()
            },
            UiGlobalTransform::from_xy(100.0, 100.0),
        )
    }

    fn spawn_joystick(app: &mut App, joystick: VirtualJoystickNode<String>) -> (Entity, Entity) {
        let world = app.world_mut();
        let (style, node, transform) = ui_geometry(100.0);
        let entity = world
            .spawn(
                VirtualJoystickBundle::new(joystick)
                    .set_style(style)
                    .set_node(node)
                    .set_global_transform(transform),
            )
            .id();
        world.spawn((
            VirtualJoystickUIBackground,
            ChildOf(entity),
            ui_geometry(100.0),
        ));
        world.spawn((VirtualJoystickUIKnob, ChildOf(entity), ui_geometry(20.0)));
        let area = world
            .spawn((
                VirtualJoystickInteractionArea,
                ChildOf(entity),
                ui_geometry(100.0),
            ))
            .id();
        (entity, area)
    }

    fn fixed_joystick() -> VirtualJoystickNode<String> {
        VirtualJoystickNode::default()
            .with_id("move".into())
            .with_behavior(JoystickFixed)
    }

    fn joystick_state(app: &App, entity: Entity) -> &VirtualJoystickState {
        app.world().get::<VirtualJoystickState>(entity).unwrap()
    }

    fn touch(app: &mut App, window: Entity, id: u64, phase: TouchPhase, position: Vec2) {
        app.world_mut()
            .write_message(TouchInput {
                phase,
                position,
                window,
                force: None,
                id,
            })
            .unwrap();
    }

    fn assert_message(
        app: &App,
        cursor: &mut Cursor,
        message_type: VirtualJoystickMessageType,
        value: Vec2,
        axis: Vec2,
    ) {
        let messages = app
            .world()
            .resource::<Messages<VirtualJoystickMessage<String>>>();
        let mut messages = cursor.read(messages);
        let message = messages.next().expect("joystick message");
        assert_eq!(message.id(), "move");
        assert_eq!(message.get_type(), message_type);
        assert_eq!(*message.value(), value);
        assert_eq!(*message.axis(), axis);
        assert!(messages.next().is_none());
    }

    fn assert_silent(app: &App, cursor: &mut Cursor) {
        let messages = app
            .world()
            .resource::<Messages<VirtualJoystickMessage<String>>>();
        assert_eq!(cursor.read(messages).count(), 0);
    }

    #[test]
    fn touch_press_drag_hold_end_and_cancel() {
        let (mut app, window) = test_app();
        let (entity, _) = spawn_joystick(&mut app, fixed_joystick());
        let mut cursor = Cursor::default();
        let center = Vec2::splat(100.0);
        let moved = Vec2::new(125.0, 75.0);

        for terminal_phase in [TouchPhase::Ended, TouchPhase::Canceled] {
            for (phase, position, message_type, axis) in [
                (TouchPhase::Started, center, Press, Vec2::ZERO),
                (TouchPhase::Moved, moved, Drag, Vec2::splat(0.5)),
            ] {
                touch(&mut app, window, 7, phase, position);
                app.update();
                assert_message(&app, &mut cursor, message_type, position, axis);
                let state = joystick_state(&app, entity);
                let pointer = state.pointer_state.as_ref().unwrap();
                assert_eq!(pointer.id, Some(7));
                assert_eq!(pointer.start, center);
                assert_eq!(pointer.current, position);
                assert_eq!(pointer.just_pressed, message_type == Press);
                assert!(!state.just_released);
            }

            app.update();
            assert_message(&app, &mut cursor, Drag, moved, Vec2::splat(0.5));
            touch(&mut app, window, 7, terminal_phase, moved);
            app.update();
            assert_message(&app, &mut cursor, Up, Vec2::ZERO, Vec2::ZERO);
            let state = joystick_state(&app, entity);
            assert!(state.pointer_state.is_none());
            assert!(state.just_released);
            for _ in 0..2 {
                app.update();
                assert_silent(&app, &mut cursor);
                assert!(!joystick_state(&app, entity).just_released);
            }
        }
    }

    #[test]
    fn mouse_press_drag_hold_and_release_without_touch_id() {
        let (mut app, window) = test_app();
        let (entity, _) = spawn_joystick(&mut app, fixed_joystick());
        let mut cursor = Cursor::default();
        let center = Vec2::splat(100.0);
        let moved = Vec2::new(75.0, 125.0);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);

        for (position, message_type, axis) in [
            (center, Press, Vec2::ZERO),
            (moved, Drag, Vec2::splat(-0.5)),
            (moved, Drag, Vec2::splat(-0.5)),
        ] {
            app.world_mut()
                .get_mut::<Window>(window)
                .unwrap()
                .set_cursor_position(Some(position));
            app.update();
            assert_message(&app, &mut cursor, message_type, position, axis);
            let pointer = joystick_state(&app, entity).pointer_state.as_ref().unwrap();
            assert_eq!(pointer.id, None);
            assert_eq!(pointer.start, center);
            assert_eq!(pointer.current, position);
            assert_eq!(pointer.just_pressed, message_type == Press);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .clear();
        }

        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        assert_message(&app, &mut cursor, Up, Vec2::ZERO, Vec2::ZERO);
        assert!(joystick_state(&app, entity).pointer_state.is_none());
        assert!(joystick_state(&app, entity).just_released);
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.update();
        assert_silent(&app, &mut cursor);
        assert!(!joystick_state(&app, entity).just_released);
    }

    #[test]
    fn missing_state_is_inserted_without_overwriting_existing_state() {
        let (mut app, _) = test_app();
        let missing = app
            .world_mut()
            .spawn(VirtualJoystickNode::<String>::default())
            .id();
        let initial = VirtualJoystickState {
            pointer_state: Some(PointerState {
                id: Some(99),
                start: Vec2::new(80.0, 90.0),
                current: Vec2::new(110.0, 120.0),
                just_pressed: false,
            }),
            just_released: false,
            base_offset: Vec2::new(12.0, 18.0),
            delta: Vec2::new(0.25, -0.5),
        };
        let existing = app
            .world_mut()
            .spawn((VirtualJoystickNode::<String>::default(), initial.clone()))
            .id();

        for _ in 0..2 {
            app.update();
            let state = joystick_state(&app, missing);
            assert!(state.pointer_state.is_none());
            assert!(!state.just_released);
            assert_eq!(state.base_offset, Vec2::ZERO);
            assert_eq!(state.delta, Vec2::ZERO);
            let state = joystick_state(&app, existing);
            assert_eq!(state.base_offset, initial.base_offset);
            assert_eq!(state.delta, initial.delta);
            assert_eq!(state.just_released, initial.just_released);
            let pointer = state.pointer_state.as_ref().unwrap();
            let expected = initial.pointer_state.as_ref().unwrap();
            assert_eq!(pointer.id, expected.id);
            assert_eq!(pointer.start, expected.start);
            assert_eq!(pointer.current, expected.current);
            assert_eq!(pointer.just_pressed, expected.just_pressed);
        }
    }

    #[derive(Resource, Default)]
    struct ActionLog(Vec<(String, VirtualJoystickMessageType, Entity, Vec2)>);

    #[derive(Component, Debug, PartialEq)]
    struct BehaviorAxis(Vec2);

    struct Recorder;

    impl VirtualJoystickBehavior for Recorder {
        fn update(&self, world: &mut World, entity: Entity) {
            let axis = world.get::<VirtualJoystickState>(entity).unwrap().delta;
            world.entity_mut(entity).insert(BehaviorAxis(axis));
        }
    }

    impl Recorder {
        fn record(
            id: String,
            kind: VirtualJoystickMessageType,
            data: VirtualJoystickState,
            world: &mut World,
            entity: Entity,
        ) {
            world
                .resource_mut::<ActionLog>()
                .0
                .push((id, kind, entity, data.delta));
        }
    }

    impl VirtualJoystickAction<String> for Recorder {
        fn on_start_drag(
            &self,
            id: String,
            data: VirtualJoystickState,
            world: &mut World,
            entity: Entity,
        ) {
            Self::record(id, Press, data, world, entity);
        }

        fn on_drag(
            &self,
            id: String,
            data: VirtualJoystickState,
            world: &mut World,
            entity: Entity,
        ) {
            Self::record(id, Drag, data, world, entity);
        }

        fn on_end_drag(
            &self,
            id: String,
            data: VirtualJoystickState,
            world: &mut World,
            entity: Entity,
        ) {
            Self::record(id, Up, data, world, entity);
        }
    }

    #[test]
    fn constraints_reach_messages_behavior_and_world_mutating_actions() {
        let (mut app, window) = test_app();
        app.init_resource::<ActionLog>();
        let (entity, _) = spawn_joystick(
            &mut app,
            fixed_joystick()
                .with_behavior((
                    JoystickFixed,
                    JoystickHorizontalOnly,
                    JoystickDeadZone(0.25),
                    Recorder,
                ))
                .with_action(Recorder),
        );
        let mut cursor = Cursor::default();
        let frames = [
            (
                TouchPhase::Started,
                Vec2::new(110.0, 75.0),
                Press,
                Vec2::ZERO,
            ),
            (
                TouchPhase::Moved,
                Vec2::new(125.0, 75.0),
                Drag,
                Vec2::new(0.5, 0.0),
            ),
            (
                TouchPhase::Moved,
                Vec2::new(112.5, 75.0),
                Drag,
                Vec2::new(0.25, 0.0),
            ),
            (TouchPhase::Ended, Vec2::new(112.5, 75.0), Up, Vec2::ZERO),
        ];

        for (index, (phase, position, kind, axis)) in frames.into_iter().enumerate() {
            touch(&mut app, window, 3, phase, position);
            app.update();
            let value = if kind == Up { Vec2::ZERO } else { position };
            assert_message(&app, &mut cursor, kind, value, axis);
            assert_eq!(
                app.world().get::<BehaviorAxis>(entity),
                Some(&BehaviorAxis(axis))
            );
            let actions = &app.world().resource::<ActionLog>().0;
            assert_eq!(actions.len(), index + 1);
            assert_eq!(actions[index], ("move".into(), kind, entity, axis));
        }
        app.update();
        assert_silent(&app, &mut cursor);
        assert_eq!(app.world().resource::<ActionLog>().0.len(), frames.len());
    }

    #[test]
    fn scaled_interaction_area_controls_capture_and_axis() {
        let (mut app, window) = test_app();
        let (entity, area) = spawn_joystick(&mut app, fixed_joystick());
        let mut cursor = Cursor::default();
        let world = app.world_mut();
        world.resource_mut::<UiScale>().0 = 2.0;
        let mut nodes = world.query::<&mut ComputedNode>();
        for mut node in nodes.iter_mut(world) {
            node.inverse_scale_factor = 0.25;
        }
        world.get_mut::<ComputedNode>(area).unwrap().size = Vec2::splat(20.0);

        let outside_area = Vec2::new(70.0, 50.0);
        touch(&mut app, window, 1, TouchPhase::Started, outside_area);
        app.update();
        assert_silent(&app, &mut cursor);
        assert!(joystick_state(&app, entity).pointer_state.is_none());
        touch(&mut app, window, 1, TouchPhase::Ended, outside_area);
        let inside_area = Vec2::new(52.5, 47.5);
        touch(&mut app, window, 2, TouchPhase::Started, inside_area);
        app.update();
        assert_message(&app, &mut cursor, Press, inside_area, Vec2::splat(0.1));
        let pointer = joystick_state(&app, entity).pointer_state.as_ref().unwrap();
        assert_eq!(pointer.id, Some(2));
        let moved = Vec2::new(150.0, 50.0);
        touch(&mut app, window, 2, TouchPhase::Moved, moved);
        app.update();
        assert_message(&app, &mut cursor, Drag, moved, Vec2::X);
    }
}
