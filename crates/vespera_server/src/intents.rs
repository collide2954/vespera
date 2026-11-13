use bevy::prelude::*;
use bevy_replicon::prelude::*;
use vespera_protocol::InteractIntent;
use vespera_rules::woodcutting::{ChopRequest, Tree};

use crate::characters::Controls;

pub struct IntentsPlugin;

impl Plugin for IntentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            forward_interactions
                .after(ServerSystems::Receive)
                .run_if(in_state(ServerState::Running)),
        );
    }
}

fn forward_interactions(
    mut intents: MessageReader<FromClient<InteractIntent>>,
    clients: Query<&Controls>,
    trees: Query<(), With<Tree>>,
    mut chops: MessageWriter<ChopRequest>,
) {
    for intent in intents.read() {
        let ClientId::Client(client) = intent.client_id else {
            continue;
        };
        let Ok(controls) = clients.get(client) else {
            continue;
        };
        if trees.contains(intent.message.target) {
            chops.write(ChopRequest {
                woodcutter: controls.0,
                tree: intent.message.target,
            });
        }
    }
}
