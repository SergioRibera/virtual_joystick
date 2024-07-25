# Bevy Virtual Joystick
![VJoystick_Fixed_Preview](https://user-images.githubusercontent.com/56278796/230562577-e173e567-5b61-402e-929d-3d3172b0da83.gif)

</br>
<p align="center">
    <img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/SergioRibera/virtual_joystick/ci.yml?label=ci&style=flat-square">
    <a href="https://crates.io/crates/virtual_joystick"><img alt="GitHub release (latest by date)" src="https://img.shields.io/crates/v/virtual_joystick"></a>
</p>

Create and use a Virtual Joystick in a UI for [bevy](https://bevyengine.org/) Game Engine.

# Versions
Aviable and compatible versions

|  bevy  | VirtualJoystick |
|--------|-----------------|
|  0.14  |      2.2.0      |
|  0.13  |      2.2.0      |
|  0.12  |      2.1.0      |
|  0.11  |      2.0.1      |
| 0.10.1 |      1.1.2      |

# Features
- Support Mouse and Touch
- Easy usage
- Multiple Joysticks on screen
- Multiple types of joystick behaviour
- Track events on Joystick (Press, Drag and Up)
- Support Axis block (Horizontal, Vertical or Both)

> **NOTE:** To compile android projects you can use [cargo-apk](https://crates.io/crates/cargo-apk) or the [docker-rust-android](https://github.com/SergioRibera/docker-rust-android) project container where you don't have to install or prepare any sdk, for more details see the readme of the mobile projects

### Axis
| Both (Default)                                                                                                                 | Horizontal                                                                                                                           | Vertical                                                                                                                           |
|--------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------|
| ![VJoystick_Fixed_Both](https://user-images.githubusercontent.com/56278796/230561082-fc8ceb4f-0d7d-47f8-b4b8-64cdf3d713b9.gif) | ![VJoystick_Fixed_Horizontal](https://user-images.githubusercontent.com/56278796/230561186-76dba677-f7c6-41b2-9ce7-5a347f5ce480.gif) | ![VJoystick_Fixed_Vertical](https://user-images.githubusercontent.com/56278796/230561212-1b2a66a2-4fc0-456a-bfbe-5d0c89e2cd3d.gif) |

### Joystick Types
| Fixed                                                                                                                          | Floating (Default)                                                                                                                | Dynamic (TODO: Fix movement feel)                                                                                                |
|--------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------------|
| ![VJoystick_Fixed_Both](https://user-images.githubusercontent.com/56278796/230561082-fc8ceb4f-0d7d-47f8-b4b8-64cdf3d713b9.gif) | ![VJoystick_Floating_Both](https://user-images.githubusercontent.com/56278796/230561292-b9bcc015-17fc-4ef5-9a65-2ce8cc69f073.gif) | ![VJoystick_Dynamic_Both](https://user-images.githubusercontent.com/56278796/230561327-3aeb4c1a-f3ee-49e4-84a9-4872f2c261e3.gif) |

# Examples
- [Mobile](./examples/simple_mobile)
- [Desktop](./examples/simple.rs)
- [Multiple Joysticks Mobile](./examples/multiple_joysticks_mobile)
- [Multiple Joysticks Desktop](./examples/multiple.rs)

# Features
- inspect: for world inspect with egui inspector
- [`serde`](https://serde.rs) (default): for serialization support for all types (usable for save and load settings)

```toml
virtual_joystick = {
    version = "*",
    default-features = false,
    features = [ "inspect", "serde" ]
}
```

# Usage
Check out the [examples](./examples) for details.

```sh
# to run example
cargo run --example simple -F=inspect
```

Add to Cargo.toml
```toml
[dependencies]
bevy = "0.12"
virtual_joystick = "*" # Add your version
```

The minimal requirement:
```rust
use bevy::prelude::*;
// import crate
use virtual_joystick::*;

// ID for joysticks
#[derive(Default, Reflect, Hash, Clone, PartialEq, Eq)]
enum JoystickControllerID {
    #[default]
    Joystick1,
    Joystick2,
}

#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add plugin to application
        .add_plugin(VirtualJoystickPlugin::<JoystickControllerID>::default())
        .run()
}
```

Create Joystick
```rust
#[bevy_main]
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Add plugin to application
        .add_plugin(VirtualJoystickPlugin)
        // Create system
        .add_startup_system(create_scene)
        // update System
        .add_system(update_player)
        .run()
}


fn create_scene(mut cmd: Commands, asset_server: Res<AssetServer>) {
    cmd.spawn(Camera2dBundle::default());
    cmd.spawn_empty().insert(Player(30.));

    // Spawn Virtual Joystick at horizontal center
    create_joystick(
        &mut cmd,
        asset_server.load("Knob.png"),
        asset_server.load("Outline.png"),
        None,
        None,
        Some(Color::rgba(1.0, 0.27, 0.0, 0.3))),
        Vec2::new(75., 75.),
        Vec2::new(150., 150.),
        VirtualJoystickNode {
            dead_zone: 0.,
            id: "UniqueJoystick".to_string(),
            axis: VirtualJoystickAxis::Both,
            behaviour: VirtualJoystickType::Floating,
        },
        Style {
            width: Val::Px(150.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            left: Val::Percent(50.),
            bottom: Val::Percent(15.),
            ..default()
        },
    );
}
```

Use variable generated by Joystick
```rust
fn update_joystick(
    mut joystick: EventReader<VirtualJoystickEvent<String>>,
    mut player: Query<(&mut Transform, &Player)>,
    time_step: Res<Time>,
) {
    // Get player
    let (mut player, player_data) = player.single_mut();

    // Iter each joystick event
    for j in joystick.read() {
        let Vec2 { x, y } = j.axis();
        // Verify ID of joystick for movement
        match j.id() {
            JoystickControllerID::Joystick1 => {
                // Move player using joystick axis value
                player.translation.x += x * player_data.0 * time_step.delta_seconds();
                player.translation.y += y * player_data.0 * time_step.delta_seconds();
            }
        }
    }
}
```

# TODOs
- [ ] WIP: Add more better documentation
