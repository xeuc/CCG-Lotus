//! Card pack opening sequence — idiomatic Bevy 0.18 ECS.
//!
//! # Design principles
//!
//! ## Component presence IS state
//! Pack carries exactly one step-marker ZST at a time (StepIntro,
//! StepOpening, …). There is no enum, no resource holding the "current
//! phase". Inserting a marker is entering a step; removing it is leaving.
//!
//! ## OnAdd observers are the entry points
//! Every animated step has a global OnAdd observer. The moment the marker
//! lands on Pack (via Commands::insert), Bevy fires the observer
//! synchronously during the next command flush. That observer spawns tweens
//! and sets PendingTweens on Pack to the count of spawned tweens.
//! Wait steps (StepWaitSwipe1, StepWaitCard) have no observer — they are
//! purely passive, unblocked only by a PlayerSwiped trigger.
//!
//! ## TweenBatchDone drives automatic advancement
//! When any tween finishes, the tick system fires a global TweenBatchDone
//! trigger (carrying the batch_id). on_tween_batch_done queries Pack, checks
//! the id, decrements PendingTweens, and inserts the next marker when it
//! reaches zero.
//!
//! ## Interruption via batch IDs
//! NextBatchId is bumped on every step entry. Stale tweens (from a skipped
//! step) fire TweenBatchDone with the old id, which no longer matches Pack's
//! CurrentBatchId, so they are silently ignored.
//!
//! ## The only three Update systems
//! tick_transform_tweens, tick_arc_tweens, dispatch_swipe.
//! Everything else is observer-driven and runs only when something changes.

use std::f32::consts::PI;

use bevy::{color::palettes::basic::*, prelude::*};
use crate::GameState;

// =============================================================================
// PLUGIN
// =============================================================================

pub struct DevPlaygroundPlugin;

impl Plugin for DevPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<SwipeEvent>()
            .init_resource::<CardProgress>()
            .init_resource::<NextBatchId>()

            // Global OnAdd observers — fire when their step marker is added to
            // any entity. In practice only Pack ever receives step markers.
            .add_observer(on_add_step_intro)
            .add_observer(on_add_step_opening)
            .add_observer(on_add_step_first_card)
            .add_observer(on_add_step_card_reveal)
            .add_observer(on_add_step_next_card)

            // Global event observers — both query Pack directly via With<Pack>
            // so no entity targeting is needed.
            .add_observer(on_tween_batch_done)
            .add_observer(on_player_swiped)

            // Chain ensures reset_resources runs before spawning,
            // which matters because NextBatchId must be 0 before StepIntro
            // triggers on_add_step_intro.
            // No apply_deferred needed: all spawns and the StepIntro insertion
            // share the same command flush. When OnAdd fires, Lid and Cards
            // already exist in the world.
            .add_systems(OnEnter(GameState::DevPlayground), (
                reset_resources,
                spawn_camera,
                spawn_return_button,
                spawn_swipe_button,
                spawn_pack_lid,
                spawn_cards,
                spawn_pack_body,   // <-- spawns Pack WITH StepIntro in the bundle
            ).chain())

            // Three generic systems that know nothing about the pack sequence.
            .add_systems(Update, (
                tick_transform_tweens,
                tick_arc_tweens,   // runs after — arc wins if both exist on same entity
                dispatch_swipe,
            )
            .chain()
            .run_if(in_state(GameState::DevPlayground)));
    }
}

// =============================================================================
// EVENTS AND TRIGGERS
// =============================================================================

/// Buffered message produced by the swipe button or a touch recogniser.
/// dispatch_swipe converts it to a PlayerSwiped global trigger.
#[derive(Message)]
pub struct SwipeEvent;

/// Global trigger fired by dispatch_swipe.
/// on_player_swiped reacts to it and advances or interrupts the sequence.
#[derive(Event)]
struct PlayerSwiped;

/// Global trigger fired by tick systems when a tween completes.
/// Carries batch_id so on_tween_batch_done can reject stale completions
/// from steps that were interrupted and skipped.
#[derive(Event)]
struct TweenBatchDone {
    batch_id: u32,
}

// =============================================================================
// STEP MARKER COMPONENTS  —  exactly one lives on Pack at a time
// =============================================================================

#[derive(Component)] struct StepIntro;       // pack + lid fly in from off-screen
#[derive(Component)] struct StepWaitSwipe1;  // waiting for first player swipe
#[derive(Component)] struct StepOpening;     // lid swings open, pack nudges
#[derive(Component)] struct StepFirstCard;   // first card rises from the pack
#[derive(Component)] struct StepWaitCard;    // waiting for card-reveal swipe
#[derive(Component)] struct StepCardReveal;  // current card flies along arc
#[derive(Component)] struct StepNextCard;    // one-frame: increment progress + route
#[derive(Component)] struct StepComplete;    // all cards revealed

// =============================================================================
// ENTITY MARKERS
// =============================================================================

#[derive(Component)] pub struct Pack;

/// Completely independent from Pack — animates on its own tween,
/// unaffected by anything happening on the Pack entity.
#[derive(Component)] pub struct Lid;

#[derive(Component)] pub struct Card { pub index: usize }

// =============================================================================
// BATCH TRACKING COMPONENTS  —  live on Pack
// =============================================================================

/// How many tweens from the current step are still running.
/// Set by each step OnAdd observer; decremented by on_tween_batch_done.
/// Advancing happens when this reaches zero.
#[derive(Component, Default)]
struct PendingTweens(u32);

/// The batch_id currently active on Pack.
/// TweenBatchDone triggers with a different id are stale and ignored.
#[derive(Component, Default)]
struct CurrentBatchId(u32);

// =============================================================================
// RESOURCES
// =============================================================================

/// Which card index is currently being (or about to be) revealed.
#[derive(Resource, Default)]
pub struct CardProgress {
    pub current: usize,
    pub total:   usize,
}

/// Monotonically increasing counter. Each animated step observer bumps this
/// once before spawning tweens, producing a unique id for that step's batch.
#[derive(Resource, Default)]
struct NextBatchId(u32);

// =============================================================================
// EASING
// =============================================================================

#[derive(Clone, Copy, Debug, Default)]
pub enum Easing {
    #[default] Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    /// Overshoots then snaps back — springy landing feel.
    EaseOutBack,
}

impl Easing {
    #[inline]
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear    => t,
            Easing::EaseIn    => t * t * t,
            Easing::EaseOut   => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOut => {
                if t < 0.5 { 4.0 * t * t * t }
                else { 1.0 - (-2.0 * t + 2.0_f32).powi(3) / 2.0 }
            }
            Easing::EaseOutBack => {
                const C1: f32 = 1.701_58;
                const C3: f32 = C1 + 1.0;
                1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
            }
        }
    }
}

// =============================================================================
// TWEEN COMPONENTS
// =============================================================================

/// Linearly interpolates Transform from start to end, then self-removes.
/// batch_id links this tween to the step that spawned it.
#[derive(Component)]
pub struct TransformTween {
    pub start:    Transform,
    pub end:      Transform,
    pub duration: f32,
    pub elapsed:  f32,
    pub easing:   Easing,
    pub batch_id: u32,
}

/// Moves an entity along a quadratic Bézier arc, then self-removes.
/// P(t) = (1-t)²·P0  +  2(1-t)t·P1  +  t²·P2
/// P1 is the control point that shapes the arc height and direction.
#[derive(Component)]
pub struct ArcTween {
    pub start:       Vec3,
    pub control:     Vec3,
    pub end:         Vec3,
    pub start_scale: Vec3,
    pub end_scale:   Vec3,
    pub duration:    f32,
    pub elapsed:     f32,
    pub batch_id:    u32,
}

impl ArcTween {
    #[inline]
    fn sample(&self, t: f32) -> Vec3 {
        let u = 1.0 - t;
        u * u * self.start + 2.0 * u * t * self.control + t * t * self.end
    }
}

// =============================================================================
// TWEEN TICK SYSTEMS  —  generic, know nothing about the pack sequence
// =============================================================================

fn tick_transform_tweens(
    mut commands: Commands,
    time:         Res<Time>,
    mut q:        Query<(Entity, &mut Transform, &mut TransformTween)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut tw) in &mut q {
        tw.elapsed = (tw.elapsed + dt).min(tw.duration);
        let t = tw.easing.apply(tw.elapsed / tw.duration);
        tf.translation = tw.start.translation.lerp(tw.end.translation, t);
        tf.rotation    = tw.start.rotation.slerp(tw.end.rotation, t);
        tf.scale       = tw.start.scale.lerp(tw.end.scale, t);
        if tw.elapsed >= tw.duration {
            *tf = tw.end; // snap to exact end — no floating-point drift
            let batch_id = tw.batch_id;
            commands.entity(entity).remove::<TransformTween>();
            commands.trigger(TweenBatchDone { batch_id });
        }
    }
}

/// Runs after tick_transform_tweens. If both tweens exist on the same entity
/// (swipe during a card rise), the arc translation writes last and wins.
fn tick_arc_tweens(
    mut commands: Commands,
    time:         Res<Time>,
    mut q:        Query<(Entity, &mut Transform, &mut ArcTween)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut tw) in &mut q {
        tw.elapsed = (tw.elapsed + dt).min(tw.duration);
        let t = Easing::EaseInOut.apply(tw.elapsed / tw.duration);
        tf.translation = tw.sample(t);
        tf.scale       = tw.start_scale.lerp(tw.end_scale, t);
        if tw.elapsed >= tw.duration {
            tf.translation = tw.end;
            tf.scale       = tw.end_scale;
            let batch_id = tw.batch_id;
            commands.entity(entity).remove::<ArcTween>();
            commands.trigger(TweenBatchDone { batch_id });
        }
    }
}

// =============================================================================
// INPUT DISPATCH
// =============================================================================

/// The only system aware that a Pack entity exists.
/// Reads buffered SwipeEvent messages and fires a global PlayerSwiped trigger.
fn dispatch_swipe(
    mut commands: Commands,
    mut swipe_r:  MessageReader<SwipeEvent>,
) {
    if swipe_r.read().count() > 0 {
        commands.trigger(PlayerSwiped);
    }
}

// =============================================================================
// STEP OnAdd OBSERVERS  —  one per animated step
// =============================================================================
//
// Pattern every observer follows:
//   1. Query Pack with single() to get (entity, transform).
//   2. Bump NextBatchId by 1 — this is the new step's batch_id.
//   3. Insert tweens on the relevant entities with that batch_id.
//   4. Set CurrentBatchId and PendingTweens on Pack.
//
// Wait steps (StepWaitSwipe1, StepWaitCard) intentionally have NO observer.
// They do nothing until a PlayerSwiped trigger arrives.

/// Step 0 — Pack and Lid fly in from off-screen simultaneously.
fn on_add_step_intro(
    _trigger:       On<Add, StepIntro>,
    mut commands:   Commands,
    mut next_batch: ResMut<NextBatchId>,
    pack_q:         Query<(Entity, &Transform), With<Pack>>,
    lid_q:          Query<(Entity, &Transform), With<Lid>>,
) {
    let Ok((pack, &pack_tf)) = pack_q.single() else { return };
    next_batch.0 += 1;
    let batch = next_batch.0;
    let mut count = 0u32;

    commands.entity(pack).insert(TransformTween {
        start: pack_tf,
        end:   Transform::from_xyz(0.0, 40.0, 0.0).with_scale(Vec3::splat(1.0)),
        duration: 0.90, elapsed: 0.0, easing: Easing::EaseOutBack, batch_id: batch,
    });
    count += 1;

    if let Ok((lid, &lid_tf)) = lid_q.single() {
        commands.entity(lid).insert(TransformTween {
            start: lid_tf,
            end:   Transform::from_xyz(0.0, 167.0, 1.0).with_scale(Vec3::splat(1.0)),
            duration: 0.90, elapsed: 0.0, easing: Easing::EaseOutBack, batch_id: batch,
        });
        count += 1;
    }

    commands.entity(pack)
        .insert(CurrentBatchId(batch))
        .insert(PendingTweens(count));
}

/// Step 2 — Lid swings open; pack gets a small inertia kick.
fn on_add_step_opening(
    _trigger:       On<Add, StepOpening>,
    mut commands:   Commands,
    mut next_batch: ResMut<NextBatchId>,
    pack_q:         Query<(Entity, &Transform), With<Pack>>,
    lid_q:          Query<(Entity, &Transform), With<Lid>>,
) {
    let Ok((pack, &pack_tf)) = pack_q.single() else { return };
    next_batch.0 += 1;
    let batch = next_batch.0;
    let mut count = 0u32;

    commands.entity(pack).insert(TransformTween {
        start: pack_tf,
        end:   Transform::from_xyz(0.0, 55.0, 0.0).with_scale(Vec3::splat(1.08)),
        duration: 0.30, elapsed: 0.0, easing: Easing::EaseOut, batch_id: batch,
    });
    count += 1;

    if let Ok((lid, &lid_tf)) = lid_q.single() {
        commands.entity(lid).insert(TransformTween {
            start: lid_tf,
            end: Transform {
                translation: Vec3::new(-30.0, 265.0, 1.0),
                rotation:    Quat::from_rotation_z(0.95), // ~54 degrees
                scale:       Vec3::splat(1.0),
            },
            duration: 0.45, elapsed: 0.0, easing: Easing::EaseOutBack, batch_id: batch,
        });
        count += 1;
    }

    commands.entity(pack)
        .insert(CurrentBatchId(batch))
        .insert(PendingTweens(count));
}

/// Step 3 — First card rises from inside the pack; pack settles down.
fn on_add_step_first_card(
    _trigger:       On<Add, StepFirstCard>,
    mut commands:   Commands,
    mut next_batch: ResMut<NextBatchId>,
    progress:       Res<CardProgress>,
    pack_q:         Query<(Entity, &Transform), With<Pack>>,
    cards_q:        Query<(Entity, &Card)>,
) {
    let Ok((pack, &pack_tf)) = pack_q.single() else { return };
    next_batch.0 += 1;
    let batch = next_batch.0;
    let mut count = 0u32;

    commands.entity(pack).insert(TransformTween {
        start: pack_tf,
        end:   Transform::from_xyz(0.0, 20.0, 0.0).with_scale(Vec3::splat(1.0)),
        duration: 0.45, elapsed: 0.0, easing: Easing::EaseOut, batch_id: batch,
    });
    count += 1;

    for (card_entity, card) in &cards_q {
        if card.index == progress.current {
            // Hard-code the start position: the card is hidden, its actual
            // Transform is irrelevant. We place it just inside the pack opening.
            commands.entity(card_entity)
                .insert(Visibility::Visible)
                .insert(TransformTween {
                    start: Transform::from_xyz(0.0, -10.0, 5.0).with_scale(Vec3::splat(0.50)),
                    end:   Transform::from_xyz(0.0,  60.0, 5.0).with_scale(Vec3::splat(1.12)),
                    duration: 0.55, elapsed: 0.0, easing: Easing::EaseOutBack, batch_id: batch,
                });
            count += 1;
            break;
        }
    }

    commands.entity(pack)
        .insert(CurrentBatchId(batch))
        .insert(PendingTweens(count));
}

/// Step 5 — Current card flies along a Bézier arc; next card peeks from pack.
/// Reads the card's live Transform so that interrupting the rise mid-animation
/// starts the arc from wherever the card currently is, not from a fixed point.
fn on_add_step_card_reveal(
    _trigger:       On<Add, StepCardReveal>,
    mut commands:   Commands,
    mut next_batch: ResMut<NextBatchId>,
    progress:       Res<CardProgress>,
    pack_q:         Query<Entity, With<Pack>>,
    cards_q:        Query<(Entity, &Transform, &Card)>,
) {
    let Ok(pack) = pack_q.single() else { return };
    let idx = progress.current;
    next_batch.0 += 1;
    let batch = next_batch.0;
    let mut count = 0u32;

    for (card_entity, &card_tf, card) in &cards_q {
        if card.index == idx {
            let p0 = card_tf.translation;
            commands.entity(card_entity).insert(ArcTween {
                start:       p0,
                control:     Vec3::new(p0.x - 80.0,  p0.y + 230.0, p0.z),
                end:         Vec3::new(p0.x + 310.0, p0.y + 130.0, p0.z),
                start_scale: card_tf.scale,
                end_scale:   Vec3::splat(0.70),
                duration: 0.55, elapsed: 0.0, batch_id: batch,
            });
            count += 1;
        }
        // Peek the next card from the pack opening in parallel with the arc.
        if card.index == idx + 1 {
            let peek_z = 4.5 + idx as f32 * 0.1;
            commands.entity(card_entity)
                .insert(Visibility::Visible)
                .insert(TransformTween {
                    start: Transform::from_xyz(0.0, -10.0, peek_z).with_scale(Vec3::splat(0.50)),
                    end:   Transform::from_xyz(0.0,  55.0, peek_z).with_scale(Vec3::splat(1.05)),
                    duration: 0.40, elapsed: 0.0, easing: Easing::EaseOut, batch_id: batch,
                });
            count += 1;
        }
    }

    commands.entity(pack)
        .insert(CurrentBatchId(batch))
        .insert(PendingTweens(count));
}

/// One-frame transit state — only reached via natural arc completion.
/// Increments progress then routes to StepWaitCard or StepComplete.
/// Swipe interrupts bypass this entirely (they handle the increment themselves
/// in on_player_swiped, then go straight to StepCardReveal).
fn on_add_step_next_card(
    _trigger:       On<Add, StepNextCard>,
    mut commands:   Commands,
    mut next_batch: ResMut<NextBatchId>,
    mut progress:   ResMut<CardProgress>,
    pack_q:         Query<Entity, With<Pack>>,
) {
    let Ok(pack) = pack_q.single() else { return };
    progress.current += 1;

    // Bump the batch even for wait/complete steps so any residual
    // TweenBatchDone from the just-finished arc is safely rejected.
    next_batch.0 += 1;
    commands.entity(pack)
        .remove::<StepNextCard>()
        .insert(CurrentBatchId(next_batch.0))
        .insert(PendingTweens(0));

    if progress.current >= progress.total {
        commands.entity(pack).insert(StepComplete);
        info!("[PackOpen] All {} cards revealed!", progress.total);
    } else {
        commands.entity(pack).insert(StepWaitCard);
    }
}













// =============================================================================
// GLOBAL EVENT OBSERVERS
// =============================================================================

/// Reacts to every TweenBatchDone trigger.
/// Checks batch_id against Pack's CurrentBatchId; rejects stale ones.
/// Decrements PendingTweens; inserts the next step marker when it hits zero.
fn on_tween_batch_done(
    trigger:      On<TweenBatchDone>,
    mut commands: Commands,
    mut pack_q:   Query<(
        Entity,
        &mut PendingTweens,
        &CurrentBatchId,
        Has<StepIntro>,
        Has<StepOpening>,
        Has<StepFirstCard>,
        Has<StepCardReveal>,
    ), With<Pack>>,
) {
    let Ok((pack, mut pending, current_batch, is_intro, is_opening, is_first_card, is_card_reveal))
        = pack_q.single_mut() else { return };

    if trigger.event().batch_id != current_batch.0 { return; } // stale — ignore
    if pending.0 == 0 { return; }

    pending.0 -= 1;
    if pending.0 > 0 { return; } // other tweens in this batch still running

    // All tweens done — advance to the next step by swapping the marker.
    // Inserting an animated step marker fires its OnAdd observer immediately
    // at the next command flush, spawning tweens for that step.
    if is_intro {
        commands.entity(pack).remove::<StepIntro>().insert(StepWaitSwipe1);
    } else if is_opening {
        commands.entity(pack).remove::<StepOpening>().insert(StepFirstCard);
    } else if is_first_card {
        commands.entity(pack).remove::<StepFirstCard>().insert(StepWaitCard);
    } else if is_card_reveal {
        // StepNextCard increments progress and decides the next destination.
        commands.entity(pack).remove::<StepCardReveal>().insert(StepNextCard);
    }
}

/// Reacts to PlayerSwiped.
/// Waiting steps advance immediately. Animated steps are interrupted:
/// for steps whose next destination has an OnAdd observer (animated steps),
/// we just insert the new marker and its observer allocates a fresh batch_id,
/// automatically orphaning the old tweens. For steps leading to passive
/// wait markers (no OnAdd observer), we bump NextBatchId manually.
fn on_player_swiped(
    _trigger:       On<PlayerSwiped>,
    mut commands:   Commands,
    mut next_batch: ResMut<NextBatchId>,
    mut progress:   ResMut<CardProgress>,
    pack_q:         Query<(
        Entity,
        Has<StepWaitSwipe1>,
        Has<StepIntro>,
        Has<StepOpening>,
        Has<StepFirstCard>,
        Has<StepWaitCard>,
        Has<StepCardReveal>,
    ), With<Pack>>,
) {
    let Ok((pack, is_wait1, is_intro, is_opening, is_first_card, is_wait_card, is_card_reveal))
        = pack_q.single() else { return };

    if is_wait1 {
        // Waiting → animated. OnAdd<StepOpening> allocates the new batch.
        commands.entity(pack).remove::<StepWaitSwipe1>().insert(StepOpening);

    } else if is_wait_card {
        // Waiting → animated. OnAdd<StepCardReveal> allocates the new batch.
        commands.entity(pack).remove::<StepWaitCard>().insert(StepCardReveal);

    } else if is_intro {
        // Skip intro. StepWaitSwipe1 has no OnAdd observer — bump manually.
        next_batch.0 += 1;
        commands.entity(pack)
            .remove::<StepIntro>()
            .insert(CurrentBatchId(next_batch.0))
            .insert(PendingTweens(0))
            .insert(StepWaitSwipe1);

    } else if is_opening {
        // Skip opening. OnAdd<StepFirstCard> will allocate a fresh batch,
        // orphaning the lid and pack tweens currently running.
        commands.entity(pack).remove::<StepOpening>().insert(StepFirstCard);
            info!("[PackOpen] Skip opening.");

    } else if is_first_card {
        // Skip first-card rise. StepWaitCard has no OnAdd observer.
        next_batch.0 += 1;
        commands.entity(pack)
            .remove::<StepFirstCard>()
            .insert(CurrentBatchId(next_batch.0))
            .insert(PendingTweens(0))
            .insert(StepWaitCard);
            info!("[PackOpen] First Card revealed!");

    } else if is_card_reveal {
        // Interrupt the arc and throw the next card immediately.
        // We handle the increment here (not via StepNextCard) so we can skip
        // the wait step and go straight back to StepCardReveal.
        progress.current += 1;
        if progress.current >= progress.total {
            next_batch.0 += 1;
            commands.entity(pack)
                .remove::<StepCardReveal>()
                .insert(CurrentBatchId(next_batch.0))
                .insert(PendingTweens(0))
                .insert(StepComplete);
            info!("[PackOpen] All {} cards revealed!", progress.total);
        } else {
            // Remove then re-insert StepCardReveal so OnAdd fires again for
            // the next card. The observer allocates a fresh batch_id,
            // orphaning the arc that was just interrupted.
            commands.entity(pack)
                .remove::<StepCardReveal>()
                .insert(StepCardReveal);
            info!("[PackOpen] Progress is {}/{}", progress.current, progress.total);
        }
    }
}

















// =============================================================================
// SCENE SETUP
// =============================================================================

fn reset_resources(
    mut progress:   ResMut<CardProgress>,
    mut next_batch: ResMut<NextBatchId>,
) {
    *progress   = CardProgress::default();
    *next_batch = NextBatchId::default();
}

fn spawn_camera(mut commands: Commands) {
    // commands.spawn((Camera2d, DespawnOnExit(GameState::DevPlayground)));
    // Spawn camera
    commands.spawn((
        DespawnOnExit(GameState::DevPlayground),
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 1000.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
}

/// Spawns Pack with StepIntro already in the bundle.
/// When this command flushes (together with all other OnEnter commands),
/// OnAdd<StepIntro> fires and finds Lid + Cards already in the world.
fn spawn_pack_body(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut materials_standard: ResMut<Assets<StandardMaterial>>,
) {
    // commands.spawn((
    //     Pack,
    //     StepIntro,
    //     PendingTweens::default(),
    //     CurrentBatchId::default(),
    //     DespawnOnExit(GameState::DevPlayground),
    //     Mesh2d(meshes.add(Rectangle::new(140.0, 200.0))),
    //     MeshMaterial2d(materials.add(Color::srgb(0.10, 0.28, 0.72))),
    //     Transform::from_xyz(0.0, 800.0, 0.0).with_scale(Vec3::splat(0.3)),
    // ));
    commands.spawn((
        Pack,
        StepIntro,
        PendingTweens::default(),
        CurrentBatchId::default(),
        DespawnOnExit(GameState::DevPlayground),

        Mesh3d(meshes.add(Cuboid::new(140.0, 200.0, 0.1))),
        MeshMaterial3d(materials_standard.add(StandardMaterial {
            unlit: true,
            base_color: Color::srgb(0.10, 0.28, 0.72),
            ..default()
        })),
        Transform::from_xyz(0.0, 800.0, 0.0).with_scale(Vec3::splat(0.3)),
    ));

}

fn spawn_pack_lid(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut materials_standard: ResMut<Assets<StandardMaterial>>,
) {
    // commands.spawn((
    //     Lid,
    //     DespawnOnExit(GameState::DevPlayground),
    //     Mesh2d(meshes.add(Rectangle::new(140.0, 55.0))),
    //     MeshMaterial2d(materials.add(Color::srgb(0.18, 0.54, 0.92))),
    //     Transform::from_xyz(0.0, 900.0, 1.0).with_scale(Vec3::splat(0.3)),
    // ));
    commands.spawn((
        Lid,
        DespawnOnExit(GameState::DevPlayground),
        Mesh3d(meshes.add(Cuboid::new(140.0, 55.0, 0.1))),
        MeshMaterial3d(materials_standard.add(StandardMaterial {
            base_color: Color::srgb(0.18, 0.54, 0.92),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 900.0, 1.0).with_scale(Vec3::splat(0.3)),
    ));
        

        
}

fn spawn_cards(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials_color: ResMut<Assets<ColorMaterial>>,
    mut progress:  ResMut<CardProgress>,

    mut animation_players: Query<(Entity, &mut AnimationPlayer)>,
    mut materials_standard: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let card_height = 20.0;
    let card_width = 14.0;
    let card_thickness = 0.1;



    let image_height = card_width - 1.;
    let image_width = card_height - 1.;
    let a = Vec3::new(0.0, 0.0, -2.0 * card_thickness);
    let b = Vec3::new(0.0, 0.0, -1.0 * card_thickness);
    let c = Vec3::new(0.0, 0.0, 0.0);
    let d = Vec3::new(0.0, 0.0, 1.0 * card_thickness);
    let e = Vec3::new(0.0, 0.0, 2.0 * card_thickness);
    let card_colors = [
        Color::srgb(0.92, 0.24, 0.39), // Ruby
        Color::srgb(0.95, 0.74, 0.08), // Gold
        Color::srgb(0.18, 0.78, 0.44), // Emerald
        Color::srgb(0.25, 0.58, 0.95), // Sapphire
        Color::srgb(0.58, 0.18, 0.88), // Amethyst
    ];


    for (i, color) in card_colors.into_iter().enumerate() {

        
                // spawn frame of card
                // commands.spawn((
                //         Card { index: i },
                //         Transform::from_xyz(0.0, 0.0, 2.0 + i as f32 * 0.1).with_scale(Vec3::splat(1.0)),
                //         Visibility::Hidden,
                //     DespawnOnExit(GameState::DevPlayground),
                //     Mesh3d(meshes.add(Cuboid::new(card_width, card_height, card_thickness))),
                //     MeshMaterial3d(
                //         materials_standard.add(StandardMaterial {
                //             base_color: color,
                //             ..default()
                //         })
                //     ),
                // ))
                // .with_children(|parent| {
                //     // For Recto and verso
                //     for a in [("textures/40921678_S1J5493BMXVBDKB3RF7P22B9N0.jpeg", 1.0, Quat::from_rotation_y(0.0)), ("textures/25973315_8HS551035DXVATFV2SADZRBG30.jpeg", -1.0, Quat::from_rotation_y(PI))] {
                //         let photo_texture = asset_server.load(a.0);
                //         parent.spawn((
                //             DespawnOnExit(GameState::DevPlayground),
                //             Mesh3d(meshes.add(Plane3d {
                //                 normal: Dir3::Z,
                //                 half_size: Vec2::new(image_height/2., image_width/2.), // 13*19
                //             })),
                //             MeshMaterial3d(materials_standard.add(StandardMaterial {
                //                 base_color_texture: Some(photo_texture),
                //                 metallic: 0.0,
                //                 perceptual_roughness: 1.0,
                //                 ..default()
                //             })),
                //             Transform::from_translation(Vec3::new(0.0, 0.0, a.1 * card_thickness / 2.0 + 0.001)).with_rotation(a.2),
                //         ));
                //     }

                // });

        commands.spawn((
            Card { index: i },
            DespawnOnExit(GameState::DevPlayground),
            // Mesh2d(meshes.add(Rectangle::new(108.0, 156.0))),
            Mesh3d(meshes.add(Cuboid::new(108.0, 156.0,0.1))),
            // MeshMaterial2d(materials_color.add(color)),
            MeshMaterial3d(materials_standard.add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(0.0, -60.0, 2.0 + i as f32 * 0.1),
            Visibility::Hidden,
        ));


    }
    progress.current = 0;
    progress.total   = card_colors.len();
}









// =============================================================================
// UI
// =============================================================================

fn spawn_return_button(mut commands: Commands) {
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

fn spawn_swipe_button(mut commands: Commands) {
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

fn set_bg_on<E: EntityEvent>(
    color: BackgroundColor,
) -> impl Fn(On<E>, Query<&mut BackgroundColor>) {
    move |event, mut query| {
        if let Ok(mut bg) = query.get_mut(event.event_target()) {
            *bg = color.clone();
        }
    }
}

fn set_game_state_on<E: EntityEvent>(
    new_state: GameState,
) -> impl Fn(On<E>, ResMut<NextState<GameState>>) {
    move |_, mut next| { next.set(new_state); }
}

fn send_swipe_on<E: EntityEvent>() -> impl FnMut(On<E>, MessageWriter<SwipeEvent>) {
    move |_, mut sw: MessageWriter<SwipeEvent>| { sw.write(SwipeEvent); }
}