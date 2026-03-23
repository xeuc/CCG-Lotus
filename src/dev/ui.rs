
use bevy::{color::palettes::basic::*, prelude::*};
use crate::{GameState, dev::components::*};



// =============================================================================
// UI
// =============================================================================

pub fn spawn_return_button(mut commands: Commands) {
    commands
        .spawn((
            Button,
            DespawnOnExit(GameState::DevPlayground),
            BackgroundColor(WHITE.into()),
            Node {
                justify_content: JustifyContent::Center,
                align_items:     AlignItems::Center,
                position_type:   PositionType::Absolute,
                left: px(50), right: px(50), bottom: px(50),
                ..default()
            },
        ))
        .observe(set_game_state_on::<Pointer<Press>>(GameState::InUI))
        .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
        .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
        .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
        .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
        .with_child((
            DespawnOnExit(GameState::DevPlayground),
            Text::new("Return to UI"),
            TextFont { font_size: 30.0, ..default() },
            TextColor::BLACK,
            TextLayout::new_with_justify(Justify::Center),
        ));
}

pub fn spawn_swipe_button(mut commands: Commands) {
    commands
        .spawn((
            Button,
            DespawnOnExit(GameState::DevPlayground),
            BackgroundColor(WHITE.into()),
            Node {
                justify_content: JustifyContent::Center,
                align_items:     AlignItems::Center,
                position_type:   PositionType::Absolute,
                left: px(50), right: px(50), bottom: px(100),
                ..default()
            },
        ))
        .observe(send_swipe_on::<Pointer<Press>>())
        .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
        .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
        .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
        .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
        .with_child((
            DespawnOnExit(GameState::DevPlayground),
            Text::new("Swipe"),
            TextFont { font_size: 30.0, ..default() },
            TextColor::BLACK,
            TextLayout::new_with_justify(Justify::Center),
        ));
}

// =============================================================================
// UI HELPERS
// =============================================================================

pub fn set_bg_on<E: EntityEvent>(
    color: BackgroundColor,
) -> impl Fn(On<E>, Query<&mut BackgroundColor>) {
    move |event, mut query| {
        if let Ok(mut bg) = query.get_mut(event.event_target()) {
            *bg = color.clone();
        }
    }
}

pub fn set_game_state_on<E: EntityEvent>(
    new_state: GameState,
) -> impl Fn(On<E>, ResMut<NextState<GameState>>) {
    move |_, mut next| { next.set(new_state); }
}

pub fn send_swipe_on<E: EntityEvent>() -> impl FnMut(On<E>, MessageWriter<SwipeEvent>) {
    move |_, mut sw: MessageWriter<SwipeEvent>| { sw.write(SwipeEvent); }
}