use bevy::{color::palettes::basic::*, prelude::*};
use crate::dev::components::*;


// =============================================================================
// TWEEN TICK SYSTEMS  —  generic, know nothing about the pack sequence
// =============================================================================

pub fn tick_transform_tweens(
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
pub fn tick_arc_tweens(
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
pub fn dispatch_swipe(
    mut commands: Commands,
    mut swipe_r:  MessageReader<SwipeEvent>,
) {
    if swipe_r.read().count() > 0 {
        commands.trigger(PlayerSwiped);
    }
}