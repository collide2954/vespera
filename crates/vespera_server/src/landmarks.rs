use bevy::prelude::*;
use bevy_replicon::prelude::*;
use vespera_content::Landmarks;
use vespera_core::Position;
use vespera_rules::woodcutting::Tree;

pub struct LandmarksPlugin;

impl Plugin for LandmarksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_landmarks);
    }
}

fn spawn_landmarks(mut commands: Commands, landmarks: Option<Res<Landmarks>>) {
    let Some(landmarks) = landmarks else {
        return;
    };
    for placement in landmarks.trees() {
        commands.spawn((
            Replicated,
            Tree(placement.tree.clone()),
            Position(placement.position),
        ));
    }
}
