//! A 3d Scene with a button and playing sound.
use bevy::{light::DirectionalLightShadowMap, prelude::*};


mod audio;
mod input;
mod setup;
#[cfg(target_os = "android")]
mod android;


// the `bevy_main` proc_macro generates the required boilerplate for Android
#[bevy_main]
pub fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins((
            DefaultPlugins,
            audio::AudioPlugin,
            input::InputPlugin,
            #[cfg(target_os = "android")]
            android::AndroidPlugin,
            setup::SetupPlugin,
        ))
        .run();
}

