use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use virtual_joystick::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: true,
        })
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(VirtualJoystickPlugin::<String>::default())
        .add_systems(Startup, create_scene)
        .add_systems(Update, update_joystick)
        .run();
}

#[derive(Component)]
/// Player with velocity
struct Player(pub f32);

struct TintAction {
    down: Color,
    up: Color,
}

impl VirtualJoystickAction<String> for TintAction {
    fn on_start_drag(
        &self,
        _id: String,
        _data: VirtualJoystickState,
        world: &mut World,
        entity: Entity,
    ) {
        let children = world.get::<Children>(entity).iter().collect::<Vec<_>>();

        for child in children.iter() {
            let Some(mut ui_image) = world.get_mut::<ImageNode>(*child) else {
                continue;
            };
            ui_image.color = self.down;
        }
    }

    fn on_end_drag(
        &self,
        _id: String,
        _data: VirtualJoystickState,
        world: &mut World,
        entity: Entity,
    ) {
        let children = world.get::<Children>(entity).iter().collect::<Vec<_>>();

        for child in children.iter() {
            let Some(mut ui_image) = world.get_mut::<ImageNode>(*child) else {
                continue;
            };
            ui_image.color = self.up;
        }
    }
}

fn create_scene(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2d);
    // Fake Player
    cmd.spawn((
        Sprite {
            image: asset_server.load("Knob.png"),
            color: Color::srgb(0.5, 0.0, 0.5), // Purple
            custom_size: Some(Vec2::new(50., 50.)),
            ..default()
        },
        Player(50.),
        Transform::default(),
    ));

    // Spawn Virtual Joystick at horizontal center using helper function
    create_joystick(
        &mut cmd,
        "UniqueJoystick".to_string(),
        asset_server.load("Knob.png"),
        asset_server.load("Outline.png"),
        Some(Color::srgba(0.0, 1.0, 0.0, 0.5)),  // Green
        Some(Color::srgba(0.0, 1.0, 0.0, 0.5)),  // Green
        Some(Color::srgba(1.0, 0.27, 0.0, 0.3)), // OrangeRed
        Vec2::new(75., 75.),
        Vec2::new(150., 150.),
        Node {
            width: Val::Px(150.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.),
            bottom: Val::Percent(15.),
            ..default()
        },
        JoystickFloating,
        TintAction {
            down: Color::srgba(1.0, 0.0, 0.0, 1.0), // Red
            up: Color::srgba(0.0, 1.0, 0.0, 0.5),   // Green
        },
    );
}

fn update_joystick(
    mut joystick: EventReader<VirtualJoystickEvent<String>>,
    mut player: Query<(&mut Transform, &Player)>,
    time_step: Res<Time>,
) {
    let Ok((mut player, player_data)) = player.single_mut() else {
        return;
    };

    for j in joystick.read() {
        let Vec2 { x, y } = j.axis();
        player.translation.x += x * player_data.0 * time_step.delta_secs();
        player.translation.y += y * player_data.0 * time_step.delta_secs();
    }
}
