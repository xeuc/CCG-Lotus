
use bevy::{color::palettes::basic::*, prelude::*};
use crate::dev::components::*;

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
pub fn on_add_step_intro(
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
        end:   Transform::from_xyz(0.0, -440.0, 0.0).with_scale(Vec3::splat(1.0)),
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
pub fn on_add_step_opening(
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
pub fn on_add_step_first_card(
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
pub fn on_add_step_card_reveal(
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
pub fn on_add_step_next_card(
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
pub fn on_tween_batch_done(
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
pub fn on_player_swiped(
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
