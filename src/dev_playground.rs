use bevy::{
    color::palettes::basic::*,
    prelude::*,
};

use crate::GameState;



pub struct DevPlaygroundPlugin;

impl Plugin for DevPlaygroundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_sub_state::<AnimationSubState>()
            .add_systems(OnEnter(GameState::DevPlayground), (
                spawn_camera,
                spawn_return_back_to_ui_button,
                spawn_swipe_button,

                init_pack_open_state,
                setup_pack_body,
                setup_pack_lid,
                setup_cards,
            ))
            .add_systems(Update, run_idle_sub_state.run_if(in_state(AnimationSubState::Idle)))
            .add_systems(Update, run_phase1_intro_sub_state.run_if(in_state(AnimationSubState::Phase1Intro)))
            .add_systems(Update, run_wait_swipe1_sub_state.run_if(in_state(AnimationSubState::WaitSwipe1)))
            .add_systems(Update, run_phase2_opening_sub_state.run_if(in_state(AnimationSubState::Phase2Opening)))
            .add_systems(Update, run_phase3_first_card_sub_state.run_if(in_state(AnimationSubState::Phase3FirstCard)))
            .add_systems(Update, run_wait_swipe_card_sub_state.run_if(in_state(AnimationSubState::WaitSwipeCard)))
            .add_systems(Update, run_phase4_card_reveal_sub_state.run_if(in_state(AnimationSubState::Phase4CardReveal)))
            .add_systems(Update, run_complete_sub_state.run_if(in_state(AnimationSubState::Complete)))

            // .add_systems(OnExit(GameState::InUI), cleanup_ui);

            .add_systems(OnEnter(GameState::OpeningPack), setup_opening_pack)
            .add_systems(OnExit(GameState::OpeningPack), cleanup_opening_pack)


            .add_message::<SwipeEvent>()
            .init_resource::<PackOpenState>()
            .add_systems(
                Update,(
                    // sequence_driver,       // step 2 – advance state machine
                    tick_transform_tweens, // step 3 – interpolate transforms
                    tick_arc_tweens,       // step 4 – interpolate arc throws
                )
                .run_if(in_state(GameState::DevPlayground))
                .chain(), // enforces this exact order every frame
            )
            ;
    }
}

fn setup_opening_pack(mut commands: Commands) {
    commands.insert_resource(OpeningPackData::default());
}
fn cleanup_opening_pack(mut commands: Commands) {
    commands.remove_resource::<OpeningPackData>();
}



























/// ui camera
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        DespawnOnExit(GameState::DevPlayground)
    ));
}



fn spawn_return_back_to_ui_button(mut commands: Commands) {
    commands
        .spawn((
            Button,
            DespawnOnExit(GameState::DevPlayground),
            BackgroundColor(WHITE.into()), // Couleur initiale explicite
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
        .observe(update_state_on::<Pointer<Press>>(GameState::InUI))
        .observe(update_background_color_on::<Pointer<Press>>(GREEN.into()))
        .observe(update_background_color_on::<Pointer<Over>>(GRAY.into()))
        .observe(update_background_color_on::<Pointer<Out>>(WHITE.into()))
        .with_child((
            DespawnOnExit(GameState::DevPlayground),
            Text::new("Return to UI"),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor::BLACK,
            TextLayout::new_with_justify(Justify::Center),
        ));
}



fn spawn_swipe_button(mut commands: Commands) {
    commands
        .spawn((
            Button,
            DespawnOnExit(GameState::DevPlayground),
            BackgroundColor(WHITE.into()), // Couleur initiale explicite
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
        .observe(send_swipe_message_on::<Pointer<Press>>())
        .observe(update_background_color_on::<Pointer<Press>>(GREEN.into()))
        .observe(update_background_color_on::<Pointer<Over>>(GRAY.into()))
        .observe(update_background_color_on::<Pointer<Out>>(WHITE.into()))
        .with_child((
            DespawnOnExit(GameState::DevPlayground),
            Text::new("Swipe"),
            TextFont {
                font_size: 30.0,
                ..default()
            },
            TextColor::BLACK,
            TextLayout::new_with_justify(Justify::Center),
        ));
}



// Helpers
fn update_background_color_on<E: EntityEvent>(
    new_color: BackgroundColor,
) -> impl Fn(On<E>, Query<&mut BackgroundColor>) {
    move |event, mut query| {
        if let Ok(mut color) = query.get_mut(event.event_target()) {
            *color = new_color.clone();
        }
    }
}
fn update_state_on<E: EntityEvent>(
    new_state: GameState,
) -> impl Fn(On<E>, ResMut<NextState<GameState>>) {
    move |_, mut next_state| {
        next_state.set(new_state);
    }
}
fn send_swipe_message_on<E: EntityEvent>(
) -> impl FnMut(On<E>, MessageWriter<SwipeEvent>) {
    move |_, mut sw:  MessageWriter<SwipeEvent>,| {
        sw.write(SwipeEvent);
    }
}




fn init_pack_open_state(
    mut state:     ResMut<PackOpenState>,
) {
    *state = PackOpenState::default();
}



fn setup_pack_body(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state:     ResMut<PackOpenState>,
) {

    // ── Pack body ─────────────────────────────────────────────────────────────
    //   Starts off-screen (y = 800) at 30 % size.
    //   Phase 1 animates it down to centre-top at full scale.
    let pack = commands
        .spawn((
            Pack,
            DespawnOnExit(GameState::DevPlayground),
            Mesh2d(meshes.add(Rectangle::new(140.0, 200.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.10, 0.28, 0.72))),
            Transform::from_xyz(0.0, 800.0, 0.0).with_scale(Vec3::splat(0.3)),
        ))
        .id();

    // Store entity handles in the shared state resource.
    state.pack        = Some(pack);
}

fn setup_pack_lid(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state:     ResMut<PackOpenState>,
) {
    // ── Lid (top flap) ────────────────────────────────────────────────────────
    //   Independent entity — NOT a child of Pack — so it can be animated
    //   completely separately. Rests at the top of the pack body after Phase 1,
    //   then rotates open in Phase 2.
    let lid = commands
        .spawn((
            Lid,
            DespawnOnExit(GameState::DevPlayground),
            Mesh2d(meshes.add(Rectangle::new(140.0, 55.0))),
            MeshMaterial2d(materials.add(Color::srgb(0.18, 0.54, 0.92))),
            Transform::from_xyz(0.0, 900.0, 1.0).with_scale(Vec3::splat(0.3)),
        ))
        .id();

    // Store entity handles in the shared state resource.
    state.lid         = Some(lid);
}


fn setup_cards(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state:     ResMut<PackOpenState>,
) {

    // ── Cards ─────────────────────────────────────────────────────────────────
    //   All cards start hidden below the visible frame.
    //   They are revealed one-by-one as the player swipes.
    let card_colors = [
        Color::srgb(0.92, 0.24, 0.39), // Ruby
        Color::srgb(0.95, 0.74, 0.08), // Gold
        Color::srgb(0.18, 0.78, 0.44), // Emerald
        Color::srgb(0.25, 0.58, 0.95), // Sapphire
        Color::srgb(0.58, 0.18, 0.88), // Amethyst
    ];

    let mut cards = Vec::with_capacity(card_colors.len());
    for (i, color) in card_colors.into_iter().enumerate() {
        let id = commands
            .spawn((
                DespawnOnExit(GameState::DevPlayground),
                Card { index: i },
                Mesh2d(meshes.add(Rectangle::new(108.0, 156.0))),
                MeshMaterial2d(materials.add(color)),
                // Slight Z-offset so cards stack correctly visually.
                Transform::from_xyz(0.0, -60.0, 2.0 + i as f32 * 0.1),
                Visibility::Hidden,
            ))
            .id();
        cards.push(id);
    }

    // Store entity handles in the shared state resource.
    state.total_cards = cards.len();
    state.cards       = cards;
}






































/// Returns `true` when every `TransformTween` and `ArcTween` with the given
/// `phase_gen_` has been removed from the ECS (i.e. all current-phase
/// animations have finished).
///
/// The tick systems call `commands.entity(e).remove::<…>()` on completion,
/// so this check is reliable one frame after the last tween finishes.
fn tweens_done_for_gen_(
    gen_:       u32,
    tweens:    &Query<&TransformTween>,
    arc_tweens: &Query<&ArcTween>,
) -> bool {
    !tweens.iter().any(|t| t.phase_gen_ == gen_)
        && !arc_tweens.iter().any(|t| t.phase_gen_ == gen_)
}

// =============================================================================
// PHASE BUILDERS
// =============================================================================

/// **Phase 1 – Pack Intro** (PackUnZoom → PackDown → PackZoom)
///
/// Pack and Lid animate from off-screen (y=800, scale=0.3) into their resting
/// positions. Both animate *simultaneously* (parallel).
///
/// Easing: EaseOutBack for a satisfying springy arrival.
fn phase1_intro(
    commands:   &mut Commands,
    state:      &mut PackOpenState,
    transforms: &Query<&Transform>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {
    let gen_ = state.phase_gen_;

    // Pack: fly down from top of screen, unzoom from 0.3 → 1.0
    tween_entity(
        commands, state.pack, transforms, gen_,
        Transform::from_xyz(0.0, 40.0, 0.0).with_scale(Vec3::splat(1.0)),
        0.90, Easing::EaseOutBack,
    );

    // Lid: same trajectory, rests at the top edge of the pack body
    // (pack body centre y=40, half-height=100, lid half-height=27 → y≈167)
    tween_entity(
        commands, state.lid, transforms, gen_,
        Transform::from_xyz(0.0, 167.0, 1.0).with_scale(Vec3::splat(1.0)),
        0.90, Easing::EaseOutBack,
    );

    state.phase_gen_ += 1;
    //newcode
    next_state.set(AnimationSubState::Phase1Intro);
}

/// **Phase 2 – Pack Opening**
///
/// Triggered by the player's first swipe.
///
/// - **Lid**: rotates open (translate up-left + rotate ~54°). Uses
///   `EaseOutBack` for a bouncy open feel.
/// - **Pack**: tiny upward nudge + slight zoom (×1.08) — simulates the
///   physical inertia / excitement of the pack being torn open.
fn phase2_opening(
    commands:   &mut Commands,
    state:      &mut PackOpenState,
    transforms: &Query<&Transform>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {

    let gen_ = state.phase_gen_;

    // Pack: inertia kick — nudge up, slight zoom
    tween_entity(
        commands, state.pack, transforms, gen_,
        Transform::from_xyz(0.0, 55.0, 0.0).with_scale(Vec3::splat(1.08)),
        0.30, Easing::EaseOut,
    );

    // Lid: swing open — translate up-left + rotate ≈ 54° around Z
    if let Some(lid) = state.lid {
        let start = current_transform(lid, transforms);
        let end = Transform {
            translation: Vec3::new(-30.0, 265.0, 1.0),
            rotation:    Quat::from_rotation_z(0.95), // 0.95 rad ≈ 54°
            scale:       Vec3::splat(1.0),
        };
        commands.entity(lid)
            .insert(TransformTween::new(start, end, 0.45, Easing::EaseOutBack, gen_));
    }

    state.phase_gen_ += 1;
    //newcode
    next_state.set(AnimationSubState::Phase2Opening);
}

/// **Phase 3 – First Card Appears**
///
/// - **Pack**: settles down slightly (recoil after opening force).
/// - **Card 0**: becomes visible and rises from inside the pack.
///
/// Parallel: both animate simultaneously.
fn phase3_first_card(
    commands:   &mut Commands,
    state:      &mut PackOpenState,
    transforms: &Query<&Transform>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {

    let gen_ = state.phase_gen_;

    // Pack: gen_tle settle-down after the opening excitement
    tween_entity(
        commands, state.pack, transforms, gen_,
        Transform::from_xyz(0.0, 20.0, 0.0).with_scale(Vec3::splat(1.0)),
        0.45, Easing::EaseOut,
    );

    // First card: unhide → rise from inside the pack
    if let Some(&card_id) = state.cards.get(state.current_card) {
        commands.entity(card_id).insert(Visibility::Visible);

        // Hard-coded start position (inside the pack) — not read from ECS
        // because the card is currently at its hidden rest position.
        let start = Transform::from_xyz(0.0, -10.0, 5.0).with_scale(Vec3::splat(0.5));
        let end   = Transform::from_xyz(0.0,  60.0, 5.0).with_scale(Vec3::splat(1.12));
        commands.entity(card_id)
            .insert(TransformTween::new(start, end, 0.55, Easing::EaseOutBack, gen_));
    }

        state.phase_gen_ += 1;
    //newcode
    next_state.set(AnimationSubState::Phase3FirstCard);
}

/// **Phase 4 / Phase 5 – Card Reveal Arc**
///
/// Throws the current card in a Bézier arc: up then sweeping right.
/// Starts from the card's **current** transform, so it works correctly even
/// when Phase 3 was interrupted and the card hasn't fully risen yet.
///
/// Simultaneously, the *next* card (if any) peeks out of the pack opening
/// (parallel animation).
fn card_reveal(
    commands:   &mut Commands,
    state:      &mut PackOpenState,
    transforms: &Query<&Transform>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {
    let idx = state.current_card;
    let card_id = match state.cards.get(idx) {
        Some(&e) => e,
        None     => return,
    };


    let gen_ = state.phase_gen_;

    // ── Bézier arc for the thrown card ────────────────────────────────────────
    // P₀: current position (interruption-safe — reads live transform)
    let card_tf = current_transform(card_id, transforms);
    let p0 = card_tf.translation;

    // P₁: control point — sweep up and slightly left for a natural arc
    let p1 = Vec3::new(p0.x - 80.0, p0.y + 230.0, p0.z);

    // P₂: upper-right end destination — card flies off toward the side
    let p2 = Vec3::new(p0.x + 310.0, p0.y + 130.0, p0.z);

    // If a stale TransformTween is still running on this card (e.g. from a
    // very fast swipe during Phase 3), the ArcTween will visually override it
    // because tick_arc_tweens runs AFTER tick_transform_tweens in the chain.
    // The stale tween will quietly self-remove when it expires.
    commands.entity(card_id).insert(ArcTween {
        start:       p0,
        control:     p1,
        end:         p2,
        start_scale: card_tf.scale,
        end_scale:   Vec3::splat(0.70),
        duration:    0.55,
        elapsed:     0.0,
        phase_gen_:   gen_,
    });

    // ── Next card peeks from the pack (parallel with arc) ─────────────────────
    if let Some(&next_id) = state.cards.get(idx + 1) {
        commands.entity(next_id).insert(Visibility::Visible);
        let peek_z = 4.5 + idx as f32 * 0.1;
        let ps = Transform::from_xyz(0.0, -10.0, peek_z).with_scale(Vec3::splat(0.50));
        let pe = Transform::from_xyz(0.0,  55.0, peek_z).with_scale(Vec3::splat(1.05));
        commands.entity(next_id)
            .insert(TransformTween::new(ps, pe, 0.40, Easing::EaseOut, gen_));
    }

        state.phase_gen_ += 1;
    //newcode
    next_state.set(AnimationSubState::Phase4CardReveal);
}

// =============================================================================
// TWEEN TICK SYSTEMS
// =============================================================================

/// Advances all `TransformTween` components each frame.
/// Writes the interpolated value to the entity's `Transform`.
/// Removes the component (and snaps to `end`) when `elapsed >= duration`.
fn tick_transform_tweens(
    mut commands: Commands,
    time:         Res<Time>,
    mut q:        Query<(Entity, &mut Transform, &mut TransformTween)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut tw) in &mut q {
        tw.elapsed = (tw.elapsed + dt).min(tw.duration);
        let et = tw.easing.apply(tw.elapsed / tw.duration);

        tf.translation = tw.start.translation.lerp(tw.end.translation, et);
        tf.rotation    = tw.start.rotation.slerp(tw.end.rotation, et);
        tf.scale       = tw.start.scale.lerp(tw.end.scale, et);

        if tw.elapsed >= tw.duration {
            *tf = tw.end; // exact snap — no floating-point drift
            commands.entity(entity).remove::<TransformTween>();
        }
    }
}

/// Advances all `ArcTween` components along their quadratic Bézier curves.
/// Removes the component when complete.
///
/// Intentionally runs *after* `tick_transform_tweens`. This means that if
/// both components exist on the same entity (e.g. during a mid-animation
/// swipe), the arc position always wins: it overwrites the linear tween's
/// output in the same frame.
fn tick_arc_tweens(
    mut commands: Commands,
    time:         Res<Time>,
    mut q:        Query<(Entity, &mut Transform, &mut ArcTween)>,
) {
    let dt = time.delta_secs();
    for (entity, mut tf, mut tw) in &mut q {
        tw.elapsed = (tw.elapsed + dt).min(tw.duration);
        let et = Easing::EaseInOut.apply(tw.elapsed / tw.duration);

        tf.translation = tw.sample(et);
        tf.scale       = tw.start_scale.lerp(tw.end_scale, et);

        if tw.elapsed >= tw.duration {
            tf.translation = tw.end;
            tf.scale       = tw.end_scale;
            commands.entity(entity).remove::<ArcTween>();
        }
    }
}

// =============================================================================
// HELPERS
// =============================================================================

/// Read the current `Transform` of `entity`.
/// Returns `Transform::IDENTITY` if the entity doesn't have one (shouldn't
/// happen in practice, but avoids unwrap panics).
#[inline]
fn current_transform(entity: Entity, q: &Query<&Transform>) -> Transform {
    q.get(entity).copied().unwrap_or(Transform::IDENTITY)
}

/// Convenience wrapper: spawn a `TransformTween` on `entity_opt`, using the
/// entity's current transform as the start point.
///
/// Does nothing if `entity_opt` is `None`.
fn tween_entity(
    commands:   &mut Commands,
    entity_opt: Option<Entity>,
    transforms: &Query<&Transform>,
    gen_:        u32,
    end:        Transform,
    duration:   f32,
    easing:     Easing,
) {
    if let Some(entity) = entity_opt {
        let start = current_transform(entity, transforms);
        commands.entity(entity)
            .insert(TransformTween::new(start, end, duration, easing, gen_));
    }
}
















































// =============================================================================
// EVENTS
// =============================================================================

/// Fired by a swipe-up gesture (or the SPACE key on desktop).
/// The sequence driver consumes this to advance "waiting" phases.
#[derive(Message)]
pub struct SwipeEvent;


// =============================================================================
// ENTITY MARKERS
// =============================================================================

/// Marks the pack body mesh.
#[derive(Component)]
pub struct Pack;

/// Marks the lid / top-flap mesh.
///
/// The Lid is an *independent* entity — it is NOT a child of `Pack` in the
/// ECS hierarchy. This lets us animate it completely separately (its own
/// TransformTween, its own phase_gen_) without affecting `Pack` or cards.
#[derive(Component)]
pub struct Lid;

/// Marks a single card, identified by its zero-based `index`.
#[derive(Component)]
pub struct Card {
    pub index: usize,
}

// =============================================================================
// EASING
// =============================================================================

/// A small set of standard easing functions used by the tween system.
#[derive(Clone, Copy, Debug, Default)]
pub enum Easing {
    #[default]
    Linear,
    /// t³ — starts slow, accelerates
    EaseIn,
    /// 1-(1-t)³ — fast start, decelerates to stop
    EaseOut,
    /// Smooth start *and* end
    EaseInOut,
    /// Overshoots slightly then snaps back — springy landing effect
    EaseOutBack,
}

impl Easing {
    /// Map a linear `t ∈ [0, 1]` to an eased value in approximately `[0, 1]`.
    /// `EaseOutBack` may momentarily exceed 1.0 (the overshoot).
    #[inline]
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear    => t,
            Easing::EaseIn    => t * t * t,
            Easing::EaseOut   => 1.0 - (1.0 - t).powi(3),
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0_f32).powi(3) / 2.0
                }
            }
            Easing::EaseOutBack => {
                // Classic back-easing: overshoots ~1.7 % before settling at 1.0
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

/// Smoothly interpolates an entity's `Transform` from `start` → `end`
/// over `duration` seconds, then self-removes.
///
/// Any number of entities can carry a `TransformTween` simultaneously.
/// The `phase_gen_` field links this tween to the phase that created it,
/// allowing safe interruption (see module-level docs).
#[derive(Component)]
pub struct TransformTween {
    pub start:     Transform,
    pub end:       Transform,
    /// Total animation time in seconds.
    pub duration:  f32,
    /// Seconds elapsed since the tween started.
    pub elapsed:   f32,
    pub easing:    Easing,
    /// gen_eration counter of the phase that spawned this tween.
    pub phase_gen_: u32,
}

impl TransformTween {
    pub fn new(
        start:     Transform,
        end:       Transform,
        duration:  f32,
        easing:    Easing,
        phase_gen_: u32,
    ) -> Self {
        Self { start, end, duration, elapsed: 0.0, easing, phase_gen_ }
    }
}

/// Moves an entity along a **quadratic Bézier arc** for a natural throw feel,
/// then self-removes.
///
/// Curve formula: `P(t) = (1−t)² P₀  +  2(1−t)t P₁  +  t² P₂`
/// where P₁ is the control point that shapes the arc height.
///
/// Both translation *and* scale are interpolated.
#[derive(Component)]
pub struct ArcTween {
    /// P₀ — start position (world space)
    pub start:       Vec3,
    /// P₁ — Bézier control point (shapes the arc; usually above the path)
    pub control:     Vec3,
    /// P₂ — end position (world space)
    pub end:         Vec3,
    pub start_scale: Vec3,
    pub end_scale:   Vec3,
    pub duration:    f32,
    pub elapsed:     f32,
    pub phase_gen_:   u32,
}

impl ArcTween {
    /// Sample the Bézier curve at `t ∈ [0, 1]`.
    #[inline]
    fn sample(&self, t: f32) -> Vec3 {
        let u = 1.0 - t;
        u * u * self.start + 2.0 * u * t * self.control + t * t * self.end
    }
}


/// Central resource that drives the entire animation sequence.
///
/// Entity handles are populated in `setup_scene` and read by the phase
/// builders throughout the sequence.
#[derive(Resource)]
pub struct PackOpenState {

    /// Bumped on every `begin_phase()` call. Tweens with a lower `phase_gen_`
    /// are "stale" — ignored by completion checks, but still animate visually.
    pub phase_gen_: u32,

    /// Index of the card currently being (or about to be) revealed.
    pub current_card: usize,

    /// Total number of cards in the pack.
    pub total_cards: usize,

    // Entity handles — set during setup_scene
    pub pack:  Option<Entity>,
    pub lid:   Option<Entity>,
    pub cards: Vec<Entity>,
}

impl Default for PackOpenState {
    fn default() -> Self {
        Self {
            phase_gen_:    0,
            current_card: 0,
            total_cards:  5,
            pack:         None,
            lid:          None,
            cards:        Vec::new(),
        }
    }
}


















#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(GameState = GameState::DevPlayground)]
#[states(scoped_entities)]
enum AnimationSubState {
    #[default]
    Idle,
    Phase1Intro,
    WaitSwipe1,
    Phase2Opening,
    Phase3FirstCard,
    WaitSwipeCard,
    Phase4CardReveal,
    Complete,
}



fn run_idle_sub_state(
    mut commands: Commands,
    mut state:    ResMut<PackOpenState>,
    mut swipe_r:  MessageReader<SwipeEvent>,
    transforms:   Query<&Transform>,
    tweens:       Query<&TransformTween>,
    arc_tweens:   Query<&ArcTween>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {
    info!("Entering run_idle_sub_state");
    // Collapse any number of swipes in this frame into one boolean.
    let swiped = swipe_r.read().count() > 0;

    // Are all tweens for the *current* phase finished?
    let gen_: u32 = state.phase_gen_;
    let done = tweens_done_for_gen_(gen_, &tweens, &arc_tweens);
    // Runs exactly once because phase1_intro immediately sets
    // state.phase = Phase1Intro via begin_phase().
    phase1_intro(&mut commands, &mut state, &transforms, next_state);
}

fn run_phase1_intro_sub_state(
    mut commands: Commands,
    mut state:    ResMut<PackOpenState>,
    mut swipe_r:  MessageReader<SwipeEvent>,
    transforms:   Query<&Transform>,
    tweens:       Query<&TransformTween>,
    arc_tweens:   Query<&ArcTween>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {
    info!("Entering run_phase1_intro_sub_state");
            // Collapse any number of swipes in this frame into one boolean.
            let swiped = swipe_r.read().count() > 0;

            // Are all tweens for the *current* phase finished?
            let gen_: u32 = state.phase_gen_;
            let done = tweens_done_for_gen_(gen_, &tweens, &arc_tweens);
            if swiped || done {
                state.phase_gen_ += 1;
                //newcode
                next_state.set(AnimationSubState::WaitSwipe1);
            }
}
fn run_wait_swipe1_sub_state(
    mut commands: Commands,
    mut state:    ResMut<PackOpenState>,
    mut swipe_r:  MessageReader<SwipeEvent>,
    transforms:   Query<&Transform>,
    tweens:       Query<&TransformTween>,
    arc_tweens:   Query<&ArcTween>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {
    info!("Entering run_wait_swipe1_sub_state");
            // Collapse any number of swipes in this frame into one boolean.
            let swiped = swipe_r.read().count() > 0;

            // Are all tweens for the *current* phase finished?
            let gen_: u32 = state.phase_gen_;
            let done = tweens_done_for_gen_(gen_, &tweens, &arc_tweens);
            if swiped {
                phase2_opening(&mut commands, &mut state, &transforms, next_state);
            }
}

fn run_phase2_opening_sub_state(
    mut commands: Commands,
    mut state:    ResMut<PackOpenState>,
    mut swipe_r:  MessageReader<SwipeEvent>,
    transforms:   Query<&Transform>,
    tweens:       Query<&TransformTween>,
    arc_tweens:   Query<&ArcTween>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {
    info!("Entering run_phase2_opening_sub_state");
            // Collapse any number of swipes in this frame into one boolean.
            let swiped = swipe_r.read().count() > 0;

            // Are all tweens for the *current* phase finished?
            let gen_: u32 = state.phase_gen_;
            let done = tweens_done_for_gen_(gen_, &tweens, &arc_tweens);
            if swiped || done {
                phase3_first_card(&mut commands, &mut state, &transforms, next_state);
            }
}
fn run_phase3_first_card_sub_state(
    mut commands: Commands,
    mut state:    ResMut<PackOpenState>,
    mut swipe_r:  MessageReader<SwipeEvent>,
    transforms:   Query<&Transform>,
    tweens:       Query<&TransformTween>,
    arc_tweens:   Query<&ArcTween>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {
    info!("Entering run_phase3_first_card_sub_state");
            // Collapse any number of swipes in this frame into one boolean.
            let swiped = swipe_r.read().count() > 0;

            // Are all tweens for the *current* phase finished?
            let gen_: u32 = state.phase_gen_;
            let done = tweens_done_for_gen_(gen_, &tweens, &arc_tweens);
            if swiped || done {
                state.phase_gen_ += 1;
                //newcode
                next_state.set(AnimationSubState::WaitSwipeCard);
            }
}
fn run_wait_swipe_card_sub_state(
    mut commands: Commands,
    mut state:    ResMut<PackOpenState>,
    mut swipe_r:  MessageReader<SwipeEvent>,
    transforms:   Query<&Transform>,
    tweens:       Query<&TransformTween>,
    arc_tweens:   Query<&ArcTween>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {
    info!("Entering run_wait_swipe_card_sub_state");
            // Collapse any number of swipes in this frame into one boolean.
            let swiped = swipe_r.read().count() > 0;

            // Are all tweens for the *current* phase finished?
            let gen_: u32 = state.phase_gen_;
            let done = tweens_done_for_gen_(gen_, &tweens, &arc_tweens);
            if swiped {
                card_reveal(&mut commands, &mut state, &transforms, next_state);
            }
}
fn run_phase4_card_reveal_sub_state(
    mut commands: Commands,
    mut state:    ResMut<PackOpenState>,
    mut swipe_r:  MessageReader<SwipeEvent>,
    transforms:   Query<&Transform>,
    tweens:       Query<&TransformTween>,
    arc_tweens:   Query<&ArcTween>,
    mut next_state: ResMut<NextState<AnimationSubState>>,
) {
    info!("Entering run_phase4_card_reveal_sub_state");
            // Collapse any number of swipes in this frame into one boolean.
            let swiped = swipe_r.read().count() > 0;

            // Are all tweens for the *current* phase finished?
            let gen_: u32 = state.phase_gen_;
            let done = tweens_done_for_gen_(gen_, &tweens, &arc_tweens);
            if done || swiped {
                state.current_card += 1;

                if state.current_card >= state.total_cards {
                    // All cards revealed — we're done.
                    state.phase_gen_ += 1;
                    //newcode
                    next_state.set(AnimationSubState::Complete);
                    info!("[PackOpen] ✓ All {} cards revealed!", state.total_cards);
                } else if swiped {
                    // Swipe interrupted the current card arc: throw the next
                    // card immediately without stopping to wait.
                    // The interrupted ArcTween will finish on its own (stale gen_).
                    card_reveal(&mut commands, &mut state, &transforms, next_state);
                } else {
                    // Arc finished naturally: wait for next swipe.
                    state.phase_gen_ += 1;
                    //newcode
                    next_state.set(AnimationSubState::WaitSwipeCard);
                }
            }
}
fn run_complete_sub_state() {
    info!("Entering run_complete_sub_state");

}


#[derive(Resource, Default)]
pub struct OpeningPackData {
    pub phase_gen_: u32,
    pub current_card: usize,
    pub total_cards: usize,
    pub pack: Option<Entity>,
    pub lid: Option<Entity>,
    pub cards: Vec<Entity>,
}