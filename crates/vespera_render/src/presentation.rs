use bevy::camera::RenderTarget;
use bevy::light::DirectionalLightShadowMap;
use bevy::prelude::*;
use vespera_client::{LocalMoveInput, LocalPlayer, Prediction};
use vespera_core::Position;
use vespera_motion::MoveInput;
use vespera_protocol::{Character, InteractIntent};
use vespera_rules::woodcutting::{Felled, REACH, Tree};

use crate::{
    MoveKey, Quality, RenderScalePlugin, SceneView, follow_camera, move_direction,
    nearest_in_reach, scene_image,
};

const MOVE_KEYS: [(KeyCode, MoveKey); 4] = [
    (KeyCode::KeyW, MoveKey::Forward),
    (KeyCode::KeyS, MoveKey::Back),
    (KeyCode::KeyA, MoveKey::Left),
    (KeyCode::KeyD, MoveKey::Right),
];
const CHARACTER_RADIUS: f32 = 0.3;
const CHARACTER_HEIGHT: f32 = 1.8;
const TRUNK_RADIUS: f32 = 0.35;
const TRUNK_HEIGHT: f32 = 4.0;
const STUMP_SCALE: f32 = 0.15;
const GROUND_SIZE: f32 = 200.0;
const SUN_ILLUMINANCE: f32 = 6_000.0;
const AMBIENT_BRIGHTNESS: f32 = 350.0;

pub struct PresentationPlugin;

#[derive(Resource)]
struct Looks {
    character: Handle<Mesh>,
    other_character: Handle<StandardMaterial>,
    local_character: Handle<StandardMaterial>,
    trunk: Handle<Mesh>,
    bark: Handle<StandardMaterial>,
}

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenderScalePlugin)
            .insert_resource(ClearColor(Color::srgb(0.20, 0.16, 0.30)))
            .insert_resource(GlobalAmbientLight {
                color: Color::srgb(0.55, 0.50, 0.75),
                brightness: AMBIENT_BRIGHTNESS,
                ..default()
            })
            .insert_resource(DirectionalLightShadowMap {
                size: Quality::LOW.shadow_map_size,
            })
            .add_systems(Startup, (prepare_looks, spawn_scene))
            .add_systems(
                Update,
                (
                    (dress_characters, dress_trees),
                    dress_local_player,
                    (read_movement_keys, interact_with_nearest_tree),
                    (place_characters, place_local_player, shape_trees),
                    follow_local_player,
                )
                    .chain(),
            );
    }
}

fn character_translation(position: Vec3) -> Vec3 {
    position + Vec3::Y * CHARACTER_HEIGHT / 2.0
}

fn tree_transform(position: Vec3, felled: bool) -> Transform {
    let height = if felled { STUMP_SCALE } else { 1.0 };
    Transform::from_translation(position + Vec3::Y * TRUNK_HEIGHT * height / 2.0)
        .with_scale(Vec3::new(1.0, height, 1.0))
}

fn prepare_looks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Looks {
        character: meshes.add(Capsule3d::new(
            CHARACTER_RADIUS,
            CHARACTER_HEIGHT - 2.0 * CHARACTER_RADIUS,
        )),
        other_character: materials.add(Color::srgb(0.85, 0.45, 0.25)),
        local_character: materials.add(Color::srgb(0.35, 0.85, 0.90)),
        trunk: meshes.add(Cylinder::new(TRUNK_RADIUS, TRUNK_HEIGHT)),
        bark: materials.add(Color::srgb(0.40, 0.28, 0.18)),
    });
}

fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let scene = images.add(scene_image());
    commands.spawn((
        Camera3d::default(),
        RenderTarget::from(scene.clone()),
        Quality::LOW.scene_camera(),
        follow_camera(Vec3::ZERO),
    ));
    commands.spawn((
        Camera2d,
        Camera {
            order: 1,
            ..default()
        },
        Quality::LOW.screen_camera(),
    ));
    commands.spawn((Sprite::from_image(scene), SceneView));
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.78, 0.58),
            illuminance: SUN_ILLUMINANCE,
            shadow_maps_enabled: true,
            ..default()
        },
        Quality::LOW.sun_shadows(),
        Transform::from_xyz(-6.0, 4.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(GROUND_SIZE, GROUND_SIZE))),
        MeshMaterial3d(materials.add(Color::srgb(0.24, 0.36, 0.20))),
    ));
}

fn dress_characters(
    mut commands: Commands,
    looks: Res<Looks>,
    characters: Query<Entity, Added<Character>>,
) {
    for entity in &characters {
        commands.entity(entity).insert((
            Mesh3d(looks.character.clone()),
            MeshMaterial3d(looks.other_character.clone()),
            Transform::default(),
            Visibility::default(),
        ));
    }
}

fn dress_local_player(
    mut commands: Commands,
    looks: Res<Looks>,
    players: Query<Entity, Added<LocalPlayer>>,
) {
    for entity in &players {
        commands
            .entity(entity)
            .insert(MeshMaterial3d(looks.local_character.clone()));
    }
}

fn dress_trees(
    mut commands: Commands,
    looks: Res<Looks>,
    trees: Query<(Entity, &Position), Added<Tree>>,
) {
    for (entity, position) in &trees {
        commands.entity(entity).insert((
            Mesh3d(looks.trunk.clone()),
            MeshMaterial3d(looks.bark.clone()),
            tree_transform(position.0, false),
            Visibility::default(),
        ));
    }
}

fn read_movement_keys(keys: Res<ButtonInput<KeyCode>>, mut input: ResMut<LocalMoveInput>) {
    let held = MOVE_KEYS
        .into_iter()
        .filter(|(code, _)| keys.pressed(*code))
        .map(|(_, key)| key);
    input.set_if_neq(LocalMoveInput(MoveInput {
        direction: move_direction(held),
        sprint: keys.pressed(KeyCode::ShiftLeft),
    }));
}

fn interact_with_nearest_tree(
    keys: Res<ButtonInput<KeyCode>>,
    players: Query<&Prediction, With<LocalPlayer>>,
    trees: Query<(Entity, &Position), (With<Tree>, Without<Felled>)>,
    mut intents: MessageWriter<InteractIntent>,
) {
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let Ok(prediction) = players.single() else {
        return;
    };
    let candidates = trees.iter().map(|(entity, position)| (entity, position.0));
    if let Some(target) = nearest_in_reach(prediction.position(), candidates, REACH) {
        intents.write(InteractIntent { target });
    }
}

fn place_characters(
    mut characters: Query<(&Position, &mut Transform), (With<Character>, Without<LocalPlayer>)>,
) {
    for (position, mut transform) in &mut characters {
        transform.translation = character_translation(position.0);
    }
}

fn place_local_player(mut players: Query<(&Prediction, &mut Transform), With<LocalPlayer>>) {
    for (prediction, mut transform) in &mut players {
        transform.translation = character_translation(prediction.position());
    }
}

fn shape_trees(mut trees: Query<(&Position, &mut Transform, Has<Felled>), With<Tree>>) {
    for (position, mut transform, felled) in &mut trees {
        transform.set_if_neq(tree_transform(position.0, felled));
    }
}

fn follow_local_player(
    players: Query<&Prediction, With<LocalPlayer>>,
    mut cameras: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(prediction) = players.single() else {
        return;
    };
    for mut camera in &mut cameras {
        *camera = follow_camera(prediction.position());
    }
}

#[cfg(test)]
mod tests {
    use bevy::camera::RenderTarget;
    use bevy::ecs::system::RunSystemOnce;
    use bevy::prelude::*;

    use super::{
        CHARACTER_HEIGHT, STUMP_SCALE, TRUNK_HEIGHT, character_translation, spawn_scene,
        tree_transform,
    };
    use crate::SceneView;

    #[test]
    fn the_screen_shows_the_image_that_the_scene_camera_renders() {
        let mut world = World::new();
        world.init_resource::<Assets<Mesh>>();
        world.init_resource::<Assets<StandardMaterial>>();
        world.init_resource::<Assets<Image>>();

        world
            .run_system_once(spawn_scene)
            .expect("the scene spawns");

        let (scene_image, scene_order) = world
            .query_filtered::<(&RenderTarget, &Camera), With<Camera3d>>()
            .single(&world)
            .map(|(target, camera)| (target.as_image().cloned(), camera.order))
            .expect("one scene camera");
        let (screen_shows_window, screen_order) = world
            .query_filtered::<(&RenderTarget, &Camera), With<Camera2d>>()
            .single(&world)
            .map(|(target, camera)| (matches!(target, RenderTarget::Window(_)), camera.order))
            .expect("one screen camera");
        let view_image = world
            .query_filtered::<&Sprite, With<SceneView>>()
            .single(&world)
            .map(|sprite| sprite.image.clone())
            .expect("one scene view");

        assert_eq!(scene_image, Some(view_image));
        assert!(screen_shows_window);
        assert!(screen_order > scene_order);
    }

    #[test]
    fn a_character_stands_on_the_ground() {
        let translation = character_translation(Vec3::new(1.0, 0.0, 2.0));

        assert_eq!(translation, Vec3::new(1.0, CHARACTER_HEIGHT / 2.0, 2.0));
    }

    #[test]
    fn a_standing_tree_rises_from_the_ground_at_full_height() {
        let transform = tree_transform(Vec3::new(3.0, 0.0, 1.0), false);

        assert_eq!(transform.scale, Vec3::ONE);
        assert_eq!(
            transform.translation,
            Vec3::new(3.0, TRUNK_HEIGHT / 2.0, 1.0)
        );
    }

    #[test]
    fn a_felled_tree_is_a_short_stump_on_the_ground() {
        let transform = tree_transform(Vec3::ZERO, true);

        assert_eq!(transform.scale.y, STUMP_SCALE);
        assert!((transform.translation.y - TRUNK_HEIGHT * STUMP_SCALE / 2.0).abs() < 1e-6);
    }
}
