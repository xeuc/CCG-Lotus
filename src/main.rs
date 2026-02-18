use bevy::prelude::*;

mod app;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(app::CCGLotusPlugin)
        .run();
}