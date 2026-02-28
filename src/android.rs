use bevy::prelude::*;


use bevy::window::AppLifecycle;
use bevy::winit::WinitSettings;

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
