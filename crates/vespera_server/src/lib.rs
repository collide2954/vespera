mod characters;
mod intents;
mod movement;
mod privacy;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use vespera_protocol::ProtocolPlugin;
use vespera_rules::woodcutting::WoodcuttingPlugin;

pub use characters::Controls;

pub struct ShardPlugins;

impl PluginGroup for ShardPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(RepliconSharedPlugin::default())
            .add(ServerPlugin::default())
            .add(ServerMessagePlugin)
            .add(ProtocolPlugin)
            .add(privacy::PrivacyPlugin)
            .add(characters::CharactersPlugin)
            .add(intents::IntentsPlugin)
            .add(movement::MovementPlugin)
            .add(WoodcuttingPlugin)
    }
}
