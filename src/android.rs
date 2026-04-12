use bevy::prelude::*;

// Platform     | Support of new Blockbench's armatures feature
// -------------|-------------------------------
// Windows/x86  | ✅ Yes
// Android/arm  | ❌ No (Gltf/glb loads, armatured elements won't show up)


use bevy::{
    color::palettes::basic::*,
    input::{gestures::RotationGesture, touch::TouchPhase},
    log::{Level, LogPlugin},
    prelude::*,
    window::{AppLifecycle, ScreenEdge, WindowMode},
    winit::WinitSettings,
};

use dev::components::CameraLocked;
use crate::dev;

pub struct AndroidPlugin;


impl Plugin for AndroidPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(WinitSettings::mobile())
            .add_systems(
                Update,
                (
                    // Only run the lifetime handler when an [`AudioSink`] component exists in the world.
                    // This ensures we don't try to manage audio that hasn't been initialized yet.
                    handle_lifetime.run_if(any_with_component::<AudioSink>),
                    // move_camera,
                ),
            )
            ;
    }
}


// Pause audio when app goes into background and resume when it returns.
// This is handled by the OS on iOS, but not on Android.
fn handle_lifetime(
    mut app_lifecycle_reader: MessageReader<AppLifecycle>,
    music_controller: Single<&AudioSink>,
) {
    for app_lifecycle in app_lifecycle_reader.read() {
        match app_lifecycle {
            AppLifecycle::Idle | AppLifecycle::WillSuspend | AppLifecycle::WillResume => {}
            AppLifecycle::Suspended => music_controller.pause(),
            AppLifecycle::Running => music_controller.play(),
        }
    }
}




fn move_camera(
    window: Single<&Window>,
    mut touch_inputs: MessageReader<TouchInput>,
    mut camera_transform: Single<&mut Transform, (With<Camera3d>, Without<CameraLocked>)>,
    mut last_position: Local<Option<Vec2>>,
    mut rotation_gestures: MessageReader<RotationGesture>,
) {
    for touch_input in touch_inputs.read() {
        if touch_input.phase == TouchPhase::Started {
            *last_position = None;
        }
        if let Some(last_position) = *last_position {
            **camera_transform = Transform::from_xyz(
                camera_transform.translation.x
                    + (touch_input.position.x - last_position.x) / window.width() * 5.0,
                camera_transform.translation.y,
                camera_transform.translation.z
                    + (touch_input.position.y - last_position.y) / window.height() * 5.0,
            )
            .looking_at(Vec3::ZERO, Vec3::Y);
        }
        *last_position = Some(touch_input.position);
    }
    // Rotation gestures only work on iOS
    for rotation_gesture in rotation_gestures.read() {
        let forward = camera_transform.forward();
        camera_transform.rotate_axis(forward, rotation_gesture.0 / 10.0);
    }
}