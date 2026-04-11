
use std::f32::consts::PI;

use bevy::{camera_controller::free_camera::FreeCamera, color::palettes::basic::*, prelude::*};
use crate::{GameState, dev::components::*};


use crate::dev::dev_playground::*;
use crate::dev::*;

// =============================================================================
// UI
// =============================================================================

pub fn spawn_buttons(mut commands: Commands) {
    commands
        .spawn((
            DespawnOnExit(GameState::DevPlayground),
            Pickable { should_block_lower: false, is_hoverable: false }, // omg this does smtg => no
            Node {
                display: Display::Flex,
                flex_direction: FlexDirection::RowReverse,
                flex_wrap: FlexWrap::WrapReverse,
                align_content: AlignContent::FlexStart,
                row_gap: px(10),
                column_gap: px(10),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|parent | {

            // Return button
            parent.spawn((
                Button,
                DespawnOnExit(GameState::DevPlayground),
                Text::new("Return"),
                TextFont { font_size: 30.0, ..default() },
                TextColor::BLACK,
                TextLayout::new_with_justify(Justify::Center),
                BackgroundColor(WHITE.into()),
            ))
            .observe(set_game_state_on::<Pointer<Press>>(GameState::InUI))
            .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
            .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
            ;


            // Swipe
            parent.spawn((
                Button,
                Text::new("Swipe?"),
                TextFont { font_size: 30.0, ..default() },
                TextColor::BLACK,
                TextLayout::new_with_justify(Justify::Center),
                BackgroundColor(WHITE.into()),
            Pickable { should_block_lower: true, is_hoverable: true },
            ))
            .observe(send_swipe_on::<Pointer<Press>>())
            .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
            .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
            ;

            // Look Right
            parent.spawn((
                Button,
                Text::new("LookRight"),
                TextFont { font_size: 30.0, ..default() },
                TextColor::BLACK,
                TextLayout::new_with_justify(Justify::Center),
                BackgroundColor(WHITE.into()),
            Pickable { should_block_lower: true, is_hoverable: true },
            ))
            .observe(look_right_on::<Pointer<Press>>())
            .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
            .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
            ;


            // Look left 
            parent.spawn((
                Button,
                Text::new("LookLeft"),
                TextFont { font_size: 30.0, ..default() },
                TextColor::BLACK,
                TextLayout::new_with_justify(Justify::Center),
                BackgroundColor(WHITE.into()),
            Pickable { should_block_lower: true, is_hoverable: true },
            ))
            .observe(look_left_on::<Pointer<Press>>())
            .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
            .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
            ;

            // Lock cam
            parent.spawn((
                Button,
                Text::new("LockCam"),
                TextFont { font_size: 30.0, ..default() },
                TextColor::BLACK,
                TextLayout::new_with_justify(Justify::Center),
                BackgroundColor(WHITE.into()),
            Pickable { should_block_lower: true, is_hoverable: true },
            ))
            .observe(lock_cam::<Pointer<Press>>())
            .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
            .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
            ;

            // UnLock cam
            parent.spawn((
                Button,
                Text::new("UnLockCam"),
                TextFont { font_size: 30.0, ..default() },
                TextColor::BLACK,
                TextLayout::new_with_justify(Justify::Center),
                BackgroundColor(WHITE.into()),
            Pickable { should_block_lower: true, is_hoverable: true },
            ))
            .observe(unlock_cam::<Pointer<Press>>())
            .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
            .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
            ;

            // print Camera info
            parent.spawn((
                Button,
                Text::new("PrintCam"),
                TextFont { font_size: 30.0, ..default() },
                TextColor::BLACK,
                TextLayout::new_with_justify(Justify::Center),
                BackgroundColor(WHITE.into()),
            Pickable { should_block_lower: true, is_hoverable: true },
            ))
            .observe(print_cam::<Pointer<Press>>())
            .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
            .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
            ;

            // pause game
            parent.spawn((
                Button,
                Text::new("PauseGame"),
                TextFont { font_size: 30.0, ..default() },
                TextColor::BLACK,
                TextLayout::new_with_justify(Justify::Center),
                BackgroundColor(WHITE.into()),
            Pickable { should_block_lower: true, is_hoverable: true },
            ))
            .observe(pause_game_on::<Pointer<Press>>())
            .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
            .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
            .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
            ;


        });
}




// =============================================================================
// UI HELPERS
// =============================================================================

// pub fn set_bg_on<E: EntityEvent>(
//     color: BackgroundColor,
// ) -> impl Fn(On<E>, Single<&mut BackgroundColor>) {
//     move |_, mut query| { **query = color.clone(); }
// }
pub fn set_bg_on<E: EntityEvent>(
    color: BackgroundColor,
) -> impl Fn(On<E>, Query<&mut BackgroundColor>) {
    move |event, mut query| {
        if let Ok(mut bg) = query.get_mut(event.event_target()) {
            *bg = color.clone();
        }
    }
}
pub fn lock_cam<E: EntityEvent>() -> impl FnMut(On<E>, Commands, Single<Entity, (With<Camera3d>, With<FreeCamera>)>) {
    move |_, mut commands, cam| {
        // commands.entity(*cam).insert(CameraLocked);
        commands.entity(*cam).remove::<FreeCamera>();
    }
}
pub fn unlock_cam<E: EntityEvent>() -> impl FnMut(On<E>, Commands, Single<Entity, (With<Camera3d>, Without<FreeCamera>)>) {
    move |_, mut commands, cam| { 
        // commands.entity(*cam).remove::<CameraLocked>();
        commands.entity(*cam).insert(FreeCamera {
            sensitivity: 0.2,
            friction: 25.0,
            walk_speed: 3.0,
            run_speed: 9.0,
            ..default()
        });
    }
}

pub fn pause_game_on<E: EntityEvent>() -> impl FnMut(On<E>, Commands, ResMut<NextState<IsPaused>>) {
    move |_, mut commands, mut next_state| {
        // create a substate pause so that you sdon't despawn all curent buttons
        // commands.spawn() a big button "unpause" at the middle of the screen
        // That will revert back the state
        // that button would have a Despawn on Exit (SUBSTATE_PAUSE) <= never done before
        next_state.set(IsPaused::Paused);
        
    
        commands.spawn((
            DespawnOnExit(IsPaused::Paused),
            Button,
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(50),
                ..default()
            },
            BackgroundColor(WHITE.into()),
            Pickable { should_block_lower: true, is_hoverable: true },
        ))
        .observe(unpause_game_on::<Pointer<Press>>())
        .observe(set_bg_on::<Pointer<Press>>(GREEN.into()))
        .observe(set_bg_on::<Pointer<Release>>(GRAY.into()))
        .observe(set_bg_on::<Pointer<Over>>(GRAY.into()))
        .observe(set_bg_on::<Pointer<Out>>(WHITE.into()))
        .with_child((
            Text::new("UNPAUSE"),
            TextFont { font_size: 30.0, ..default() },
            TextColor::BLACK,
            TextLayout::new_with_justify(Justify::Center),
        ));
    }
}


pub fn unpause_game_on<E: EntityEvent>() -> impl FnMut(On<E>, ResMut<NextState<IsPaused>>) {
    move |_, mut next_state| {
        // create a substate pause so that you sdon't despawn all curent buttons
        // commands.spawn() a big button "unpause" at the middle of the screen
        // That will revert back the state
        // that button would have a Despawn on Exit (SUBSTATE_PAUSE) <= never done before
        next_state.set(IsPaused::Running);
        
    }
}

pub fn print_cam<E: EntityEvent>() -> impl FnMut(On<E>, Single<&Transform, With<Camera3d>>) {
    move |_, cam_trans_quer| {
        info!(cam_trans_quer.translation.x);
        info!(cam_trans_quer.translation.y);
        info!(cam_trans_quer.translation.z);
        info!(cam_trans_quer.rotation.x);
        info!(cam_trans_quer.rotation.y);
        info!(cam_trans_quer.rotation.z);
        info!(cam_trans_quer.scale.x);
        info!(cam_trans_quer.scale.y);
        info!(cam_trans_quer.scale.z);
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


pub fn look_left_on<E: EntityEvent>() -> impl FnMut(On<E>, Single<&mut Transform, With<Camera3d>>) {
    move |_, mut cam_trans_quer| { cam_trans_quer.rotate_y(PI/2.0) }
}

pub fn look_right_on<E: EntityEvent>() -> impl FnMut(On<E>, Single<&mut Transform, With<Camera3d>>) {
    move |_, mut cam_trans_quer| { cam_trans_quer.rotate_y(-PI/2.0) }
}