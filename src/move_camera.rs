// use bevy::{input::mouse::MouseMotion, prelude::*};

// // =========================
// // Plugin
// // =========================

// pub struct CameraControllerPlugin;

// impl Plugin for CameraControllerPlugin {
//     fn build(&self, app: &mut App) {
//         app
//             .init_resource::<MouseLookState>()
//             .add_systems(
//                 Update,
//                 (
//                     camera_look_system,
//                     camera_move_system,
//                 ),
//             );
//     }
// }

// // =========================
// // Config
// // =========================

// const MOVE_SPEED: f32 = 8.0;
// const FAST_MULTIPLIER: f32 = 2.0;
// const MOUSE_SENSITIVITY: f32 = 0.0025;


// // =========================
// // Mouse state
// // =========================

// #[derive(Resource, Default)]
// struct MouseLookState {
//     yaw: f32,
//     pitch: f32,
// }

// // =========================
// // Mouse look (rotation)
// // =========================

// fn camera_look_system(
//     mut mouse_events: MessageReader<MouseMotion>,
//     mut query: Query<&mut Transform, With<Camera3d>>,
//     mut look: ResMut<MouseLookState>,
// ) {
//     let Ok(mut transform) = query.single_mut() else { return };

//     let mut delta = Vec2::ZERO;
//     for ev in mouse_events.read() {
//         delta += ev.delta;
//     }

//     if delta == Vec2::ZERO {
//         return;
//     }

//     look.yaw -= delta.x * MOUSE_SENSITIVITY;
//     look.pitch -= delta.y * MOUSE_SENSITIVITY;

//     // prevent flipping
//     look.pitch = look.pitch.clamp(-1.54, 1.54);

//     transform.rotation =
//         Quat::from_axis_angle(Vec3::Y, look.yaw)
//         * Quat::from_axis_angle(Vec3::X, look.pitch);
// }

// // =========================
// // Keyboard movement
// // =========================

// fn camera_move_system(
//     time: Res<Time>,
//     keyboard: Res<ButtonInput<KeyCode>>,
//     mut query: Query<&mut Transform, With<Camera3d>>,
//     mut app_exit_events: MessageWriter<AppExit>,
// ) {
//     // sorry lazy
//     if keyboard.pressed(KeyCode::Escape) {
//         info!("LotusDebug - Exiting app due to Escape key press");
//         app_exit_events.write(AppExit::Success);
//     }

//     let Ok(mut transform) = query.single_mut() else { return };

//     let mut input = Vec3::ZERO;

//     // Forward / backward
//     if keyboard.pressed(KeyCode::KeyW) {
//         input.z -= 1.0;
//     }
//     if keyboard.pressed(KeyCode::KeyS) {
//         input.z += 1.0;
//     }

//     // Left / right
//     if keyboard.pressed(KeyCode::KeyA) {
//         input.x -= 1.0;
//     }
//     if keyboard.pressed(KeyCode::KeyD) {
//         input.x += 1.0;
//     }

//     // Vertical
//     if keyboard.pressed(KeyCode::KeyF) {
//         input.y -= 1.0;
//     }
//     if keyboard.pressed(KeyCode::ShiftLeft) {
//         input.y += 1.0;
//     }

//     if input == Vec3::ZERO {
//         return;
//     }

//     input = input.normalize();

//     let speed = if keyboard.pressed(KeyCode::ControlLeft) {
//         MOVE_SPEED * FAST_MULTIPLIER
//     } else {
//         MOVE_SPEED
//     };

//     let forward = transform.forward();
//     let right = transform.right();
//     let up = Vec3::Y;

//     let movement =
//         forward * -input.z +
//         right * input.x +
//         up * input.y;

//     transform.translation += movement * speed * time.delta_secs();
// }
