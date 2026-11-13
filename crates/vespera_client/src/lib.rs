mod prediction;
mod transport;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use vespera_protocol::{Possess, ProtocolPlugin};

pub use prediction::{LocalMoveInput, Prediction};
pub use transport::{ConnectError, connect};

pub struct ClientLogicPlugins;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LocalPlayer;

struct PossessionPlugin;

impl PluginGroup for ClientLogicPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(RepliconSharedPlugin::default())
            .add(ClientPlugin)
            .add(ClientMessagePlugin)
            .add(ProtocolPlugin)
            .add(PossessionPlugin)
            .add(prediction::PredictionPlugin)
    }
}

impl Plugin for PossessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            possess
                .after(ClientSystems::Receive)
                .run_if(in_state(ClientState::Connected)),
        );
    }
}

fn possess(mut possessions: MessageReader<Possess>, mut commands: Commands) {
    for possession in possessions.read() {
        commands.entity(possession.character).insert(LocalPlayer);
    }
}
