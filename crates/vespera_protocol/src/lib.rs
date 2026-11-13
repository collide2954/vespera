use bevy::ecs::entity::MapEntities;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use serde::{Deserialize, Serialize};
use vespera_core::Position;
use vespera_motion::{MoveInput, SIMULATION_HZ};
use vespera_rules::pack::Pack;
use vespera_rules::skills::Skills;
use vespera_rules::woodcutting::{Felled, Tree};

pub struct ProtocolPlugin;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Character;

#[derive(Component, Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct ServerMotion {
    pub sequence: u32,
    pub position: Vec3,
    pub velocity: Vec3,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoveCommand {
    pub sequence: u32,
    pub input: MoveInput,
}

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
        app.insert_resource(Time::<Fixed>::from_hz(SIMULATION_HZ))
            .replicate::<Character>()
            .replicate::<Position>()
            .replicate::<ServerMotion>()
            .replicate::<Pack>()
            .replicate::<Skills>()
            .replicate::<Tree>()
            .replicate::<Felled>()
            .add_client_message::<MoveCommand>(Channel::Ordered)
            .add_mapped_client_message::<InteractIntent>(Channel::Ordered)
            .add_mapped_server_message::<Possess>(Channel::Ordered);
    }
}
