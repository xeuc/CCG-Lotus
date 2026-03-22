// From https://bevy.org/examples/gltf/load-gltf/
// From https://bevy.org/examples/animation/animated-mesh/

use bevy::{
    color::palettes::basic::*,
    prelude::*,
    light::CascadeShadowConfigBuilder,
};

use std::{f32::consts::*, time::Duration};

use bevy_tweening::{Lens, Tween, TweenAnim, TweeningPlugin};

use crate::{GameState, MenuButton};


// An example asset that contains a mesh and animation.
// const GLTF_PATH: &str = "models/card_pack.gltf";
// const GLTF_PATH: &str = "models/test_cube_animation.glb";
// const GLTF_PATH: &str = "models/GenerickPack2.glb";
const GLTF_PATH: &str = "models/GenerickPack4.gltf";
// const CARD_PATH: &str = "models/card_base_model.gltf";
const _CUBE_PATH_16: &str = "models/cubeScale16.gltf";
const _CUBE_PATH_01: &str = "models/cubeScale1.gltf";
const _CUBE_PATH_08: &str = "models/cubeScale8.gltf";
const TEST_ARMATURE_PATH: &str = "models/blender_armature_pack.gltf";
// const TEST_ARMATURE_PATH: &str = "models/test_armature.gltf";

const _PACK_POS: Vec3 = Vec3::new(0.0, 0.0, 0.0);
const CARP_POS: Vec3 = Vec3::new(0.0, 0.0, 0.0);


pub struct OpenCardPlugin;

impl Plugin for OpenCardPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(TweeningPlugin)
            .add_systems(OnEnter(GameState::OpeningPack), (
                spawn_camera,
                spawn_light,
                spawn_card_pack,
                spawn_return_back_to_ui_button,
                // _spawn_cube,
            ))
            .add_systems(Update, (
                setup_scene_once_loaded,
                spawn_cards,
                button_handler,
            ).run_if(in_state(GameState::OpeningPack)))
            // .add_systems(OnExit(GameState::OpeningPack), cleanup_ui);
            ;
    }
}

fn _spawn_cube(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Blockbench Scale 01
    commands.spawn((
        DespawnOnExit(GameState::OpeningPack),
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(_CUBE_PATH_01),)),
        Transform::from_translation(vec3(-0.5, 0.0, 0.0)),
    ));

    // Bevy Cube
    commands.spawn((
        DespawnOnExit(GameState::OpeningPack),
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                // alpha_mode: AlphaMode::Mask(0.5),
                // metallic: 0.0,
                // perceptual_roughness: 1.0,
                ..default()
            })
        ),
        // RotateY,
        Transform::from_translation(vec3(0.5, 0.5, 0.0)),
        // TweenAnim::new(tween3),
    ));
}




fn spawn_camera(mut commands: Commands) {
    // Spawn camera
    commands.spawn((
        DespawnOnExit(GameState::OpeningPack),
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 40.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
}

fn spawn_light(mut commands: Commands) {
    // Spawn light
    commands.spawn((
        DespawnOnExit(GameState::OpeningPack),
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This is a relatively small scene, so use tighter shadow
        // cascade bounds than the default for better quality.
        // We also adjusted the shadow map to be larger since we're
        // only using a single cascade.
        CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .build(),
    ));
}


#[derive(Resource)]
struct Animations {
    animations: AnimationNodeIndex,
    graph_handle: Handle<AnimationGraph>,
}


fn spawn_card_pack(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials_color: ResMut<Assets<ColorMaterial>>,

    mut animation_players: Query<(Entity, &mut AnimationPlayer)>,
    mut materials_standard: ResMut<Assets<StandardMaterial>>,
) {
    // Build the animation graph
    let (graph, node_indices) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(TEST_ARMATURE_PATH)),
    );

    // Keep our animation graph in a Resource so that it can be inserted onto
    // the correct entity once the scene actually loads.
    let graph_handle = graphs.add(graph);
    commands.insert_resource(Animations {
        animations: node_indices,
        graph_handle,
    });

    // Fox
    // commands.spawn((
    //     DespawnOnExit(GameState::OpeningPack),
    //     SceneRoot(
    //         asset_server.load(GltfAssetLabel::Scene(0).from_asset(TEST_ARMATURE_PATH)),
    //     ),
    //     Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
    //     .with_rotation(
    //         Quat::from_rotation_y(PI/2.0)
    //     )
    //     .with_scale(Vec3::splat(0.5))
    //     ,
    // ));

    let card_height = 20.0;
    let card_width = 14.0;
    let card_thickness = 0.1;



    let image_height = card_width - 1.;
    let image_width = card_height - 1.;
            
                // spawn frame of card
                commands.spawn((
                        Transform::from_xyz(0.0, 0.0, 2.0 + 0.0 as f32 * 0.1),
                        Visibility::Hidden,
                    DespawnOnExit(GameState::DevPlayground),
                    Mesh3d(meshes.add(Cuboid::new(card_width, card_height, card_thickness))),
                    MeshMaterial3d(
                        materials_standard.add(StandardMaterial {
                            base_color: Color::srgb(0.92, 0.24, 0.39),
                            ..default()
                        })
                    ),
                ))
                .with_children(|parent| {
                    // For Recto and verso
                    for a in [("textures/40921678_S1J5493BMXVBDKB3RF7P22B9N0.jpeg", 1.0, Quat::from_rotation_y(0.0)), ("textures/25973315_8HS551035DXVATFV2SADZRBG30.jpeg", -1.0, Quat::from_rotation_y(PI))] {
                        let photo_texture = asset_server.load(a.0);
                        parent.spawn((
                            DespawnOnExit(GameState::DevPlayground),
                            Mesh3d(meshes.add(Plane3d {
                                normal: Dir3::Z,
                                half_size: Vec2::new(image_height/2., image_width/2.), // 13*19
                            })),
                            MeshMaterial3d(materials_standard.add(StandardMaterial {
                                base_color_texture: Some(photo_texture),
                                metallic: 0.0,
                                perceptual_roughness: 1.0,
                                ..default()
                            })),
                            Transform::from_translation(Vec3::new(0.0, 0.0, a.1 * card_thickness / 2.0 + 0.001)).with_rotation(a.2),
                        ));
                    }

                });

}


// An `AnimationPlayer` is automatically added to the scene when it's ready.
// When the player is added, start the animation.
fn setup_scene_once_loaded(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations, Duration::ZERO)
            .repeat()
            ;

        commands
            .entity(entity)
            .insert(AnimationGraphHandle(animations.graph_handle.clone()))
            .insert(transitions);
    }
}


fn spawn_cards(
    mut animation_players: Query<(Entity, &mut AnimationPlayer)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (entity, mut player) in &mut animation_players {
        let Some((&playing_animation_index, _)) = player.playing_animations().next() else {
            continue;
        };

        let playing_animation = player.animation_mut(playing_animation_index).unwrap();
        if playing_animation.is_finished() {

            commands.entity(entity).remove::<AnimationPlayer>();

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
            for _pos in [a+CARP_POS, b+CARP_POS, c+CARP_POS, d+CARP_POS, e+CARP_POS] {
                
                let tween3 = Tween::new(
                    EaseFunction::CubicInOut,
                    Duration::from_secs_f32(10.0),
                    BezierPositionLens {
                        p0: Vec3::new(0.0, -1.5, -4.0) * 5.0 + a,
                        p1: Vec3::new(0.0, -0.8, -4.5) * 5.0 + b,
                        p2: Vec3::new(0.0,  0.5, -6.5) * 5.0 + c,
                        p3: Vec3::new(0.0,  0.0,  0.5) * 5.0 + d,
                    },
                );

                // spawn frame of card
                commands.spawn((
                    DespawnOnExit(GameState::OpeningPack),
                    Mesh3d(meshes.add(Cuboid::new(card_width, card_height, card_thickness))),
                    MeshMaterial3d(
                        materials.add(StandardMaterial {
                            base_color: Color::WHITE,
                            ..default()
                        })
                    ),
                    TweenAnim::new(tween3),
                ))
                .with_children(|parent| {
                    // For Recto and verso
                    for a in [("textures/40921678_S1J5493BMXVBDKB3RF7P22B9N0.jpeg", 1.0, Quat::from_rotation_y(0.0)), ("textures/25973315_8HS551035DXVATFV2SADZRBG30.jpeg", -1.0, Quat::from_rotation_y(PI))] {
                        let photo_texture = asset_server.load(a.0);
                        parent.spawn((
                            DespawnOnExit(GameState::OpeningPack),
                            Mesh3d(meshes.add(Plane3d {
                                normal: Dir3::Z,
                                half_size: Vec2::new(image_height/2., image_width/2.), // 13*19
                            })),
                            MeshMaterial3d(materials.add(StandardMaterial {
                                base_color_texture: Some(photo_texture),
                                metallic: 0.0,
                                perceptual_roughness: 1.0,
                                ..default()
                            })),
                            Transform::from_translation(Vec3::new(0.0, 0.0, a.1 * card_thickness / 2.0 + 0.001)).with_rotation(a.2),
                        ));
                    }

                });

            }

        } else {
        }
        
    }
}



fn spawn_return_back_to_ui_button(
    mut commands: Commands,
) {
    // spawn button
    commands
        .spawn((
            Button,
            MenuButton::InUI,
            DespawnOnExit(GameState::OpeningPack),
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
            DespawnOnExit(GameState::OpeningPack),
            Text::new("Return back to UI"),
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
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                // *color = BLUE.into();
                *color = GREEN.into();
                // Launch the opening!
                next_state.set(GameState::InUI);
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



pub struct BezierPositionLens {
    pub p0: Vec3,
    pub p1: Vec3,
    pub p2: Vec3,
    pub p3: Vec3,
}

impl Lens<Transform> for BezierPositionLens {
    fn lerp(&mut self, mut target: Mut<'_, bevy::prelude::Transform>, ratio: f32) {
        let t = ratio;
        let u = 1.0 - t;

        // cubic bezier
        let pos =
            u*u*u * self.p0 +
            3.0*u*u*t * self.p1 +
            3.0*u*t*t * self.p2 +
            t*t*t * self.p3;

        target.translation = pos;
    }
}
