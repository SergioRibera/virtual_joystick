use bevy::{prelude::*, ui::update};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use virtual_joystick::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(VirtualJoystickPlugin::<String>::default())
        .add_systems(Startup, create_scene)
        .add_systems(Update, update_joystick)
        .run();
}

#[derive(Component)]
// Player with velocity
struct Player(pub f32);

struct TintAction {
    down: Color,
    up: Color
}

impl VirtualJoystickAction<String> for TintAction {
    fn on_start_drag(
        &self,
        _id: String,
        _data: VirtualJoystickState,
        world: &mut World,
        entity: Entity,
    ) {
        let mut child_entities: Vec<Entity> = Vec::new();
        {
            let Some(children) = world.get::<Children>(entity) else { return; };
            for &child in children.iter() {
                child_entities.push(child);
            }
        }
        for &child in &child_entities {
            let is_base_or_knob: bool;
            {
                is_base_or_knob = world.get::<VirtualJoystickUIBackground>(entity).is_some() || world.get::<VirtualJoystickUIKnob>(entity).is_some();
            }
            let Some(mut bg_color) = world.get_mut::<BackgroundColor>(child) else { continue; };
            bg_color.0 = self.down;
        }
    }

    fn on_end_drag(
        &self,
        _id: String,
        _data: VirtualJoystickState,
        world: &mut World,
        entity: Entity,
    ) {
        let mut child_entities: Vec<Entity> = Vec::new();
        {
            let Some(children) = world.get::<Children>(entity) else { return; };
            for &child in children.iter() {
                child_entities.push(child);
            }
        }
        for &child in &child_entities {
            let is_base_or_knob: bool;
            {
                is_base_or_knob = world.get::<VirtualJoystickUIBackground>(entity).is_some() || world.get::<VirtualJoystickUIKnob>(entity).is_some();
            }
            let Some(mut bg_color) = world.get_mut::<BackgroundColor>(child) else { continue; };
            bg_color.0 = self.up;
        }
    }
}

fn create_scene(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2dBundle {
        transform: Transform::from_xyz(0., 0., 5.0),
        ..default()
    });
    // Fake Player
    cmd.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0., 0., 0.),
            ..default()
        },
        texture: asset_server.load("Knob.png"),
        sprite: Sprite {
            color: Color::PURPLE,
            custom_size: Some(Vec2::new(50., 50.)),
            ..default()
        },
        ..default()
    })
    .insert(Player(50.));

    // Spawn Virtual Joystick at horizontal center using helper function
    create_joystick(
        &mut cmd,
        "UniqueJoystick".to_string(),
        asset_server.load("Knob.png"),
        asset_server.load("Outline.png"),
        None,
        None,
        Some(Color::ORANGE_RED.with_a(0.2)),
        Vec2::new(75., 75.),
        Vec2::new(150., 150.),
        Style {
            width: Val::Px(150.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.),
            bottom: Val::Percent(15.),
            ..default()
        },
        JoystickFloating,
        TintAction {
            down: Color::RED.with_a(1.0),
            up: Color::GREEN.with_a(0.5),
        }
    );
}

fn update_joystick(
    mut joystick: EventReader<VirtualJoystickEvent<String>>,
    mut player: Query<(&mut Transform, &Player)>,
    time_step: Res<Time>,
) {
    let (mut player, player_data) = player.single_mut();

    for j in joystick.read() {
        let Vec2 { x, y } = j.axis();
        player.translation.x += x * player_data.0 * time_step.delta_seconds();
        player.translation.y += y * player_data.0 * time_step.delta_seconds();
    }
}
