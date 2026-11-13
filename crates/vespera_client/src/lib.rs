use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy_replicon::prelude::*;

pub struct ClientLogicPlugins;

impl PluginGroup for ClientLogicPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(RepliconSharedPlugin::default())
            .add(ClientPlugin)
            .add(ClientMessagePlugin)
    }
}
