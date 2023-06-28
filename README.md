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
- [Desktop](./examples/simple_pc)
- [Multiple Joysticks Mobile](./examples/multiple_joysticks_mobile)
- [Multiple Joysticks Desktop](./examples/multiple_joysticks_pc)

# Features
- inspect: for world inspect with egui inspector
- serialize (default): for serialization support for all types (usable for save and load settings)

```toml
virtual_joystick = {
    version = "*",
    default-features = false,
    features = [ "inspect", "serialize" ]
}
```

# Usage
Check out the [examples](./examples) for details.

Add to Cargo.toml
```toml
[dependencies]
bevy = "0.10.1"
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
    cmd.spawn(
        // Define variable for Joystick
        VirtualJoystickBundle::new(VirtualJoystickNode {
            border_image: asset_server.load("Outline.png"),
            knob_image: asset_server.load("Knob.png"),
            knob_size: Vec2::new(80., 80.),
            dead_zone: 0.,
            id: JoystickControllerID::Joystick1,
            axis: VirtualJoystickAxis::Horizontal,
            behaviour: VirtualJoystickType::Fixed,
        })
        .set_color(TintColor(Color::WHITE))
        .set_style(Style {
            size: Size::all(Val::Px(150.)),
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Percent(50.),
                bottom: Val::Percent(15.),
                ..default()
            },
            ..default()
        }),
    )
    // When you add this component you mark this area as interactable for Joystick
    .insert(VirtualJoystickInteractionArea);
}
```

Use variable generated by Joystick
```rust

fn update_joystick(
    mut joystick: EventReader<VirtualJoystickEvent<JoystickControllerID>>,
    mut player: Query<(&mut Transform, &Player)>,
    time_step: Res<FixedTime>,
) {
    // Get player
    let (mut player, player_data) = player.single_mut();

    // Iter each joystick event
    for j in joystick.iter() {
        // get axis value 0-1 in x & y
        let Vec2 { x, y } = j.axis();
        // Verify ID of joystick for movement
        match j.id() {
            JoystickControllerID::Joystick1 => {
                // Move player using joystick axis value
                player.translation.x += x * player_data.0 * time_step.period.as_secs_f32();
                player.translation.y += y * player_data.0 * time_step.period.as_secs_f32();
            }
        }
    }
}
```

# Types

```rust
enum VirtualJoystickAxis {
    Both, // Default
    Horizontal,
    Vertical,
}

enum VirtualJoystickType {
    /// Static position
    Fixed,
    /// Spawn at point click
    /// Default
    Floating,
    /// Follow point on drag
    Dynamic,
}

// Component
struct VirtualJoystickNode {
    /// Identifier of joystick
    /// Note: any type that implements Hash + Clone + Default + Reflect
    pub id: S,
    /// Image for background or border image on joystick
    pub border_image: Handle<Image>,
    /// Image for handler knob on joystick
    pub knob_image: Handle<Image>,
    /// Size for knob on joystick
    pub knob_size: Vec2,
    /// Zone to ignore movement
    pub dead_zone: f32,
    /// Define Axis for this joystick
    pub axis: VirtualJoystickAxis,
    /// Define the behaviour of joystick
    pub behaviour: VirtualJoystickType,
}

// Event Type
pub enum VirtualJoystickEventType {
    Press,
    Drag,
    Up
}

// EventReader
struct VirtualJoystickEvent {
    /// Get ID of joystick throw event
    pub fn id() -> S;

    /// Return the Type of Joystick Event
    pub fn get_type() -> VirtualJoystickEventType;

    /// Raw position of point (Mouse or Touch)
    pub fn value() -> Vec2;

    /// Axis of Joystick see [crate::VirtualJoystickAxis]
    pub fn direction() -> VirtualJoystickAxis;

    /// Delta value ranging from 0 to 1 in each vector (x and y)
    pub fn axis() -> Vec2;

    /// Delta value snaped
    pub fn snap_axis(dead_zone: Option<f32>) -> Vec2;
}

// Bundle to spawn
struct VirtualJoystickBundle {
    pub fn new(joystick: VirtualJoystickNode) -> Self;

    pub fn set_node(mut self, node: Node) -> Self;

    pub fn set_style(mut self, style: Style) -> Self;

    pub fn set_color(mut self, color: TintColor) -> Self;

    pub fn set_focus_policy(mut self, focus_policy: FocusPolicy) -> Self;

    pub fn set_transform(mut self, transform: Transform) -> Self;

    pub fn set_global_transform(mut self, global_transform: GlobalTransform) -> Self;

    pub fn set_visibility(mut self, visibility: Visibility) -> Self;

    pub fn set_computed_visibility(mut self, computed_visibility: ComputedVisibility) -> Self;

    pub fn set_z_index(mut self, z_index: ZIndex) -> Self;
}
```

# TODOs
- Add more better documentation
