// From https://bevy.org/examples/gltf/load-gltf/
// From https://bevy.org/examples/animation/animated-mesh/


use std::f32::consts::*;

use bevy::{light::{CascadeShadowConfigBuilder, DirectionalLightShadowMap}, prelude::*, scene::SceneInstanceReady};

pub struct CCGLotusPlugin;

impl Plugin for CCGLotusPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(DirectionalLightShadowMap { size: 4096 })
            .add_systems(Startup, setup)
            .add_systems(Startup, spawn_card)
            .add_systems(Update, rotate_x)
            .add_systems(Update, rotate_y)
            .add_systems(Update, rotate_z)
        ;

    }
}


// An example asset that contains a mesh and animation.
const GLTF_PATH: &str = "models/card_pack.gltf";
const CARD_PATH: &str = "models/card_base_model.gltf";

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    // Spawn camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 5.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        // EnvironmentMapLight {
        //     diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
        //     specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
        //     intensity: 250.0,
        //     ..default()
        // },
    ));

    // Spawn light
    commands.spawn((
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

    // Create an animation graph containing a single animation. We want the "run"
    // animation from our example asset, which has an index of two.
    let (graph, index) = AnimationGraph::from_clip(
        asset_server.load(GltfAssetLabel::Animation(0).from_asset(GLTF_PATH)),
    );

    // Store the animation graph as an asset.
    let graph_handle = graphs.add(graph);

    // Create a component that stores a reference to our animation.
    let animation_to_play = AnimationToPlay {
        graph_handle,
        index,
    };

    // Start loading the asset as a scene and store a reference to it in a
    // SceneRoot component. This component will automatically spawn a scene
    // containing our mesh once it has loaded.
    // let mesh_scene = SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(GLTF_PATH)));

    // Spawn an entity with our components, and connect it to an observer that
    // will trigger when the scene is loaded and spawned.
    // Spawn card pack model with animation
    commands.spawn((
        animation_to_play,
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(GLTF_PATH),)),
        RotateX,
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, FRAC_PI_2, 0.0, 0.0))
            .with_translation(vec3(2.0, 0.0, 0.0))
    ))
    .observe(play_animation_when_ready)
    ;

    // Spawn card model
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(CARD_PATH),)),
        RotateZ,
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 0.0, FRAC_PI_2))
            .with_translation(vec3(-2.0, 0.0, 0.0))
    ));

}


fn spawn_card(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    // let image_with_default_sampler = asset_server.load_with_settings(
    //     // "textures\\red_border.png",
    //     "textures\\rainbow_border.png",
    //     |s: &mut ImageLoaderSettings| {
    //         *s = ImageLoaderSettings {
    //             sampler: ImageSampler::Descriptor(ImageSamplerDescriptor {
    //                 mag_filter: bevy::image::ImageFilterMode::Nearest,
    //                 min_filter: bevy::image::ImageFilterMode::Nearest,
    //                 mipmap_filter: bevy::image::ImageFilterMode::Nearest,
    //                 address_mode_u: ImageAddressMode::ClampToEdge,
    //                 address_mode_v: ImageAddressMode::ClampToEdge,
    //                 ..default()
    //             }),
    //             ..default()
    //         };
    //     },
    // );

    let card_thickness = 0.064;
        
    // spawn frame of card
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.4, 2.0, card_thickness))),
        MeshMaterial3d(
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                // alpha_mode: AlphaMode::Mask(0.5),
                // metallic: 0.0,
                // perceptual_roughness: 1.0,
                ..default()
            })
        ),
        RotateY,
        Transform::from_translation(Vec3::ZERO),
    ))

    // spawn frame of card
    // commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(1.4, 2.0, 0.064))),
    //     MeshMaterial3d(materials.add(StandardMaterial {
    //         base_color_texture: Some(image_with_default_sampler.clone()),
    //         alpha_mode: AlphaMode::Mask(0.5),
    //         metallic: 0.0,
    //         perceptual_roughness: 1.0,
    //         ..default()
    //     })),
    //     // RotateZ,
    //     Transform::from_translation(Vec3::ZERO),
    // ))
    .with_children(|parent| {

        
        // spawn recto image
        let photo_texture = asset_server.load("textures/40921678_S1J5493BMXVBDKB3RF7P22B9N0.jpeg");
        parent.spawn((
            Mesh3d(meshes.add(Plane3d {
                normal: Dir3::Z,
                half_size: Vec2::new(0.65, 0.95), // 13*19
            })),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(photo_texture),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                ..default()
            })),
            // RotateZ,
            Transform::from_translation(Vec3::new(0.0, 0.0, card_thickness / 2.0 + 0.001)),
        ));
        
        // spawn verso image
        let photo_texture = asset_server.load("textures/25973315_8HS551035DXVATFV2SADZRBG30.jpeg");
        parent.spawn((
            Mesh3d(meshes.add(Plane3d {
                normal: Dir3::Z,
                half_size: Vec2::new(0.65, 0.95), // 13*19
            })),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color_texture: Some(photo_texture),
                metallic: 0.0,
                perceptual_roughness: 1.0,
                ..default()
            })),
            // RotateZ,
            Transform::from_translation(Vec3::new(0.0, 0.0, -card_thickness / 2.0 - 0.001)).with_rotation(Quat::from_rotation_y(PI)), // Don't care about z-fighting
        ));
    });



}

fn play_animation_when_ready(
    scene_ready: On<SceneInstanceReady>,
    mut commands: Commands,
    children: Query<&Children>,
    animations_to_play: Query<&AnimationToPlay>,
    mut players: Query<&mut AnimationPlayer>,
) {
    // The entity we spawned in `setup_mesh_and_animation` is the trigger's target.
    // Start by finding the AnimationToPlay component we added to that entity.
    if let Ok(animation_to_play) = animations_to_play.get(scene_ready.entity) {
        // The SceneRoot component will have spawned the scene as a hierarchy
        // of entities parented to our entity. Since the asset contained a skinned
        // mesh and animations, it will also have spawned an animation player
        // component. Search our entity's descendants to find the animation player.
        for child in children.iter_descendants(scene_ready.entity) {
            if let Ok(mut player) = players.get_mut(child) {
                // Tell the animation player to start the animation and keep
                // repeating it.
                //
                // If you want to try stopping and switching animations, see the
                // `animated_mesh_control.rs` example.
                player.play(animation_to_play.index).repeat();

                // Add the animation graph. This only needs to be done once to
                // connect the animation player to the mesh.
                commands
                    .entity(child)
                    .insert(AnimationGraphHandle(animation_to_play.graph_handle.clone()))
                    ;
            }
        }
    }
}


fn _animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_secs() * PI / 5.0,
            -FRAC_PI_4,
        );
    }
}

fn rotate_x(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<RotateX>>,
) {
    for mut transform in &mut query {
        transform.rotation *= Quat::from_rotation_x(time.delta_secs() * PI / 2.0);
    }
}

fn rotate_y(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<RotateY>>,
) {
    for mut transform in &mut query {
        transform.rotation *= Quat::from_rotation_y(time.delta_secs() * PI / 2.0);
    }
}

fn rotate_z(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<RotateZ>>,
) {
    for mut transform in &mut query {
        transform.rotation *= Quat::from_rotation_z(time.delta_secs() * PI / 2.0);
    }
}

#[derive(Component)]
struct RotateX;

#[derive(Component)]
struct RotateY;

#[derive(Component)]
struct RotateZ;


// A component that stores a reference to an animation we want to play. This is
// created when we start loading the mesh (see `setup_mesh_and_animation`) and
// read when the mesh has spawned (see `play_animation_once_loaded`).
#[derive(Component)]
struct AnimationToPlay {
    graph_handle: Handle<AnimationGraph>,
    index: AnimationNodeIndex,
}
