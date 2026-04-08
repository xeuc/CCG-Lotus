use std::f32::consts::PI;

use bevy::prelude::*;
use crate::{GameState, dev::components::*};



// =============================================================================
// SCENE SETUP
// =============================================================================

pub fn reset_resources(
    mut progress:   ResMut<CardProgress>,
    mut next_batch: ResMut<NextBatchId>,
) {
    *progress   = CardProgress::default();
    *next_batch = NextBatchId::default();
}



pub fn spawn_camera(mut commands: Commands) {
    // commands.spawn((Camera2d, DespawnOnExit(GameState::DevPlayground)));
    // Spawn camera
    commands.spawn((
        DespawnOnExit(GameState::DevPlayground),
        CameraLocked,
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: std::f32::consts::FRAC_PI_2,
            ..default()
        }),
        Transform::from_xyz(-10.0, 35.0, 10.0),
    ));
}


pub fn spawn_light(mut commands: Commands) {
    // Spawn light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(0.0, 35.0, 0.0),
    ));
}



#[derive(Resource)]
struct Animations {
    animations: AnimationNodeIndex,
    graph_handle: Handle<AnimationGraph>,
}



/// Spawns Pack with StepIntro already in the bundle.
/// When this command flushes (together with all other OnEnter commands),
/// OnAdd<StepIntro> fires and finds Lid + Cards already in the world.
pub fn spawn_pack_body(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
    mut materials_standard: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    // mut materials_color: ResMut<Assets<ColorMaterial>>,
    // mut animation_players: Query<(Entity, &mut AnimationPlayer)>,
) {
    const TEST_ARMATURE_PATH: &str = "models/blender_armature_pack.gltf";
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


    commands.spawn((
        Pack,
        StepIntro,
        PendingTweens::default(),
        CurrentBatchId::default(),
        DespawnOnExit(GameState::DevPlayground),
        // SceneRoot(
        //     asset_server.load(GltfAssetLabel::Scene(0).from_asset(TEST_ARMATURE_PATH)),
        // ),
        Mesh3d(meshes.add(Cuboid::new(140.0, 200.0, 0.1))),
        MeshMaterial3d(materials_standard.add(StandardMaterial {
            unlit: true,
            base_color: Color::srgb(0.10, 0.28, 0.72),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, 0.0)
        .with_rotation(
            Quat::from_rotation_y(PI/2.0)
        )
        .with_scale(Vec3::splat(0.3))
        ,

    ));

}


pub fn spawn_scene(
    mut commands:  Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneRoot(asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/Scene.gltf"),
    )));
}

