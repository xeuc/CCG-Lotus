
use bevy::prelude::*;

use crate::GameState;
use crate::dev::components::*;
use crate::dev::observer::*;
use crate::dev::startup::*;
use crate::dev::update::*;
use crate::dev::ui::*;

pub struct DevPlaygroundPlugin;

impl Plugin for DevPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<SwipeEvent>()
            .init_resource::<CardProgress>()
            .init_resource::<NextBatchId>()
            .add_sub_state::<IsPaused>()
            // Global OnAdd observers — fire when their step marker is added to
            // any entity. In practice only Pack ever receives step markers.
            // .add_observer(on_add_step_intro)
            // .add_observer(on_add_step_opening)
            // .add_observer(on_add_step_first_card)
            // .add_observer(on_add_step_card_reveal)
            // .add_observer(on_add_step_next_card)

            // Global event observers — both query Pack directly via With<Pack>
            // so no entity targeting is needed.
            // .add_observer(on_tween_batch_done)
            // .add_observer(on_player_swiped)

            // Chain ensures reset_resources runs before spawning,
            // which matters because NextBatchId must be 0 before StepIntro
            // triggers on_add_step_intro.
            // No apply_deferred needed: all spawns and the StepIntro insertion
            // share the same command flush. When OnAdd fires, Lid and Cards
            // already exist in the world.
            .add_systems(OnEnter(GameState::DevPlayground), (
                spawn_observers,
                reset_resources,
                spawn_camera,
                spawn_light,
                // Buttons ui
                spawn_buttons,
                // spawn_pack_lid,
                // spawn_cards,
                spawn_scene,
                spawn_pack_body,   // <-- spawns Pack WITH StepIntro in the bundle
            ).chain())

            // Three generic systems that know nothing about the pack sequence.
            .add_systems(Update, (
                tick_transform_tweens,
                tick_arc_tweens,   // runs after — arc wins if both exist on same entity
                dispatch_swipe,
                move_paper,
            )
            .chain()
            .run_if(in_state(GameState::DevPlayground)))
            // .add_systems(OnExit(GameState::DevPlayground), despawn_observers)
            ;
    }
}


fn move_paper(
    mut query: Query<(&Name, &mut Transform)>,
) {
    for (name, mut transform) in &mut query {
        if name.as_str() == "Packs" {
            transform.rotate(Quat::from_rotation_z(0.01));
        }
    }
}


fn spawn_observers(
    mut commands: Commands,
) {
    commands.spawn((
        DespawnOnExit(GameState::DevPlayground),
        Observer::new(on_add_step_intro),
    ));
    commands.spawn((
        DespawnOnExit(GameState::DevPlayground),
        Observer::new(on_add_step_opening),
    ));
    commands.spawn((
        DespawnOnExit(GameState::DevPlayground),
        Observer::new(on_add_step_first_card),
    ));
    commands.spawn((
        DespawnOnExit(GameState::DevPlayground),
        Observer::new(on_add_step_card_reveal),
    ));
    commands.spawn((
        DespawnOnExit(GameState::DevPlayground),
        Observer::new(on_add_step_next_card),
    ));
    commands.spawn((
        DespawnOnExit(GameState::DevPlayground),
        Observer::new(on_tween_batch_done),
    ));
    commands.spawn((
        DespawnOnExit(GameState::DevPlayground),
        Observer::new(on_player_swiped),
    ));
}


// In this case, instead of deriving `States`, we derive `SubStates`
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
// And we need to add an attribute to let us know what the source state is
// and what value it needs to have. This will ensure that unless we're
// in [`AppState::InGame`], the [`IsPaused`] state resource
// will not exist.
#[source(GameState = GameState::DevPlayground)]
#[states(scoped_entities)]
pub enum IsPaused {
    #[default]
    Running,
    Paused,
}

