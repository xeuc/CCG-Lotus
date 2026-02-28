use bevy::prelude::*;

use bevy::{
    color::palettes::basic::*,
    input::{gestures::RotationGesture, touch::TouchPhase},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                touch_camera,
                button_handler,
            ))
            ;
    }
}

// INPUT SYSTEMS
fn button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // *color = BLUE.into();
                *color = GREEN.into();
            }
            Interaction::Hovered => {
                *color = GRAY.into();
            }
            Interaction::None => {
                *color = WHITE.into();
            }
        }
    }
}

fn touch_camera(
    window: Query<&Window>,
    mut touch_inputs: MessageReader<TouchInput>,
    mut camera_transform: Single<&mut Transform, With<Camera3d>>,
    mut last_position: Local<Option<Vec2>>,
    mut rotation_gestures: MessageReader<RotationGesture>,
) {
    let Ok(window) = window.single() else {
        return;
    };

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