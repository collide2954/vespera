use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

pub struct ShardPlugins;

impl PluginGroup for ShardPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(RepliconSharedPlugin::default())
            .add(ServerPlugin::default())
            .add(ServerMessagePlugin)
    }
}
