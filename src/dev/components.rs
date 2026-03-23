use bevy::{color::palettes::basic::*, prelude::*};


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
pub struct PlayerSwiped;

/// Global trigger fired by tick systems when a tween completes.
/// Carries batch_id so on_tween_batch_done can reject stale completions
/// from steps that were interrupted and skipped.
#[derive(Event)]
pub struct TweenBatchDone {
    pub(crate) batch_id: u32,
}

// =============================================================================
// STEP MARKER COMPONENTS  —  exactly one lives on Pack at a time
// =============================================================================

#[derive(Component)] pub struct StepIntro;       // pack + lid fly in from off-screen
#[derive(Component)] pub struct StepWaitSwipe1;  // waiting for first player swipe
#[derive(Component)] pub struct StepOpening;     // lid swings open, pack nudges
#[derive(Component)] pub struct StepFirstCard;   // first card rises from the pack
#[derive(Component)] pub struct StepWaitCard;    // waiting for card-reveal swipe
#[derive(Component)] pub struct StepCardReveal;  // current card flies along arc
#[derive(Component)] pub struct StepNextCard;    // one-frame: increment progress + route
#[derive(Component)] pub struct StepComplete;    // all cards revealed

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
pub struct PendingTweens(pub u32);

/// The batch_id currently active on Pack.
/// TweenBatchDone triggers with a different id are stale and ignored.
#[derive(Component, Default)]
pub struct CurrentBatchId(pub u32);

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
pub struct NextBatchId(pub(crate) u32);

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
    pub fn sample(&self, t: f32) -> Vec3 {
        let u = 1.0 - t;
        u * u * self.start + 2.0 * u * t * self.control + t * t * self.end
    }
}