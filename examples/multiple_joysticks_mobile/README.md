# Bevy Virtual Joystick Mobile
This show how works on android

|  Screenshot  |  Video  |
|--------------|---------|
| ![Screenshot_2023-04-16-04-06-56-411_com sergioribera multiple_joysticks](https://user-images.githubusercontent.com/56278796/232283881-051f2b14-ce64-454c-b25a-1f81c41ab854.jpg) | [![Preview](https://user-images.githubusercontent.com/56278796/232283881-051f2b14-ce64-454c-b25a-1f81c41ab854.jpg)](https://user-images.githubusercontent.com/56278796/232283980-976b1633-2b4e-49cb-b1c0-5730d29e5384.mp4 "Preview Mobile") |

# Features
- Multiple joysticks
- Fixed Joystick
- Orientation based on sensor

> **NOTE:** The color in area is only for make it visible

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
