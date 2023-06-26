# Bevy Virtual Joystick Mobile
This show how works on android

|  Screenshot  |  Video  |
|--------------|---------|
| ![Screenshot_2023-04-07-19-05-15-360_com sergioribera simple](https://user-images.githubusercontent.com/56278796/230692998-76a0d88f-7e38-4f82-b935-1579d08bdfa9.jpg) | [![Preview](https://user-images.githubusercontent.com/56278796/230692998-76a0d88f-7e38-4f82-b935-1579d08bdfa9.jpg)](https://user-images.githubusercontent.com/56278796/230693203-a4aee07d-62ad-4a4d-b95c-f969edad7f70.mp4 "Preview Mobile") |

# Features
- Float Joystick
- Expansive area to interact with joystick

> **NOTE:** The color in area is only for make it visible

# Settings of this example
- VirtualJoystickAxis::Both
- VirtualJoystickType::Floating

# Run example
```sh
cargo apk run
```
Or use this [proyect](https://github.com/SergioRibera/docker-rust-android) for compile gradle proyect

> **NOTE:** This is the best option to have full control over the android part of the game, and it is also very useful because of the easy integration by making pipelines in github action

```sh
# on root of proyect
docker run --rm -it -v "$(pwd)/:/src" -w /src/examples/multiple_joysticks_mobile/android sergioribera/rust-android:170-sdk-33 assembleDebug
# make own
sudo chown -R $USER examples/multiple_joysticks_mobile/android/build/
# and install
adb install examples/multiple_joysticks_mobile/android/build/outputs/apk/debug/android-debug.apk
```
