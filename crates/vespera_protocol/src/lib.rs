use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use vespera_core::Position;
use vespera_rules::pack::Pack;
use vespera_rules::skills::Skills;
use vespera_rules::woodcutting::{Felled, Tree};

pub struct ProtocolPlugin;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Character;

#[derive(Message, MapEntities, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InteractIntent {
    #[entities]
    pub target: Entity,
}

#[derive(Message, MapEntities, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Possess {
    #[entities]
    pub character: Entity,
}

impl Plugin for ProtocolPlugin {
    fn build(&self, app: &mut App) {
        app.replicate::<Character>()
            .replicate::<Position>()
            .replicate::<Pack>()
            .replicate::<Skills>()
            .replicate::<Tree>()
            .replicate::<Felled>()
            .add_mapped_client_message::<InteractIntent>(Channel::Ordered)
            .add_mapped_server_message::<Possess>(Channel::Ordered);
    }
}
