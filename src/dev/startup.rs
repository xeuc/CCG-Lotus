use std::f32::consts::PI;

use bevy::{color::palettes::basic::*, prelude::*};
use crate::{GameState, dev::components::*};


use bevy::{
    color::palettes::basic::*,
    prelude::*,
    light::CascadeShadowConfigBuilder,
};

use std::{f32::consts::*, time::Duration};

use bevy_tweening::{Lens, Tween, TweenAnim, TweeningPlugin};


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
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 500.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
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
    
    mut materials_color: ResMut<Assets<ColorMaterial>>,

    mut animation_players: Query<(Entity, &mut AnimationPlayer)>,
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

pub fn spawn_pack_lid(
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
        Transform::from_xyz(0.0, 0.0, 1.0).with_scale(Vec3::splat(0.3)),
    ));
        

        
}

pub fn spawn_cards(
    mut commands:  Commands,
    mut meshes:    ResMut<Assets<Mesh>>,
    mut materials_color: ResMut<Assets<ColorMaterial>>,
    mut progress:  ResMut<CardProgress>,

    mut animation_players: Query<(Entity, &mut AnimationPlayer)>,
    mut materials_standard: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {

    let card_thickness = 0.1;

    // let card_colors = [
    //     Color::srgb(0.92, 0.24, 0.39), // Ruby
    //     Color::srgb(0.95, 0.74, 0.08), // Gold
    //     Color::srgb(0.18, 0.78, 0.44), // Emerald
    //     Color::srgb(0.25, 0.58, 0.95), // Sapphire
    //     Color::srgb(0.58, 0.18, 0.88), // Amethyst
    //     Color::srgb(0.58, 0.18, 0.88), // Amethyst
    //     Color::srgb(0.58, 0.18, 0.88), // Amethyst
    //     Color::srgb(0.58, 0.18, 0.88), // Amethyst
    //     Color::srgb(0.58, 0.18, 0.88), // Amethyst
    //     Color::srgb(0.58, 0.18, 0.88), // Amethyst
    // ];
    let n = 15;
    let card_colors: Vec<Color> = (0..n)
        .map(|i| {
            let hue = (i as f32 / n as f32) * 360.0;
            Color::hsl(hue, 0.7, 0.5) // saturation, lightness
        })
        .collect();


    for (i, color) in card_colors.into_iter().enumerate() {

        
        // spawn frame of card
        commands.spawn((
            Card { index: i },
            DespawnOnExit(GameState::DevPlayground),
            Visibility::Hidden,
            Transform::from_xyz(0.0, -460.0, 2.0 + i as f32 * 0.1),
            Mesh3d(meshes.add(Cuboid::new(108.0, 156.0,0.1))),
            MeshMaterial3d(materials_standard.add(StandardMaterial {
                base_color: color,
                unlit: true,
                ..default()
            })),
        ))
        .with_children(|parent| {
            // For Recto and verso
            for a in [("textures/40921678_S1J5493BMXVBDKB3RF7P22B9N0.jpeg", 1.0, Quat::from_rotation_y(0.0)), ("textures/25973315_8HS551035DXVATFV2SADZRBG30.jpeg", -1.0, Quat::from_rotation_y(PI))] {
                let photo_texture = asset_server.load(a.0);
                parent.spawn((
                    DespawnOnExit(GameState::DevPlayground),
                    Mesh3d(meshes.add(Plane3d {
                        normal: Dir3::Z,
                        half_size: Vec2::new(98.0/2., 146.0/2.), // 13*19
                    })),
                    MeshMaterial3d(materials_standard.add(StandardMaterial {
                        base_color_texture: Some(photo_texture),
                        unlit: true,
                        metallic: 0.0,
                        perceptual_roughness: 1.0,
                        ..default()
                    })),
                    Transform::from_translation(Vec3::new(0.0, 0.0, a.1 * card_thickness / 2.0 + 0.001)).with_rotation(a.2),
                ));
            }

        })
        ;

        // commands.spawn((
        //     Card { index: i },
        //     DespawnOnExit(GameState::DevPlayground),
        //     // Mesh2d(meshes.add(Rectangle::new(108.0, 156.0))),
        //     Mesh3d(meshes.add(Cuboid::new(108.0, 156.0,0.1))),
        //     // MeshMaterial2d(materials_color.add(color)),
        //     MeshMaterial3d(materials_standard.add(StandardMaterial {
        //         base_color: color,
        //         unlit: true,
        //         ..default()
        //     })),
        //     Transform::from_xyz(0.0, -60.0, 2.0 + i as f32 * 0.1),
        //     Visibility::Hidden,
        // ));


    }
    progress.current = 0;
    // progress.total   = card_colors.len();
    progress.total   = n;
}

