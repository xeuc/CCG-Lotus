//user_interface.rs
// previously ui.rs but wasn't recognized by vsc extention

use bevy::{
    color::palettes::basic::*,
    prelude::*,
};

use crate::{GameState, MenuButton};

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(GameState::InUI), (
                spawn_camera,
                spawn_test_button,
                spawn_dev_playground_button,
            ))
            .add_systems(Update, (
                button_handler,
            ).run_if(in_state(GameState::InUI)))
            // .add_systems(OnExit(GameState::InUI), cleanup_ui);
            ;
    }
}


// SETUP

/// ui camera
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        DespawnOnExit(GameState::InUI)
    ));
}

/// set up a simple 3D scene
fn spawn_test_button(
    mut commands: Commands,
) {

    info!("LotusDebug - Setting up scene...");
    // spawn button test ui
    commands
        .spawn((
            Button,
            MenuButton::OpenPack,
            DespawnOnExit(GameState::InUI),
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                left: px(50),
                right: px(50),
                bottom: px(50),
                ..default()
            },
        ))
        .with_child((
            DespawnOnExit(GameState::InUI),
            Text::new("Open Pack"),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor::BLACK,
            TextLayout::new_with_justify(Justify::Center),
        ));
}

/// set up a simple 3D scene
fn spawn_dev_playground_button(
    mut commands: Commands,
) {

    // spawn button to go to dev playground state
    commands
        .spawn((
            Button,
            MenuButton::DevPlayground,
            DespawnOnExit(GameState::InUI),
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                left: px(50),
                right: px(50),
                bottom: px(100),
                ..default()
            },
        ))
        .with_child((
            DespawnOnExit(GameState::InUI),
            Text::new("Dev Playground"),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor::BLACK,
            TextLayout::new_with_justify(Justify::Center),
        ));
}

// Update
fn button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &MenuButton),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color, button) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // *color = BLUE.into();
                *color = GREEN.into();
                // Launch the opening!
                let new_state = match button {
                    MenuButton::OpenPack => GameState::OpeningPack,
                    MenuButton::DevPlayground => GameState::DevPlayground,
                    MenuButton::InUI => GameState::InUI,
                };
                info!("LotusDebug - Button pressed, changing state to {:?}", new_state);
                next_state.set(new_state);
                info!("LotusDebug - Current state: {:?}", next_state);
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