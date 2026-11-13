use bevy::prelude::*;
use bevy_replicon::prelude::*;
use vespera_core::Position;
use vespera_protocol::{Character, Possess, ServerMotion};
use vespera_rules::pack::Pack;
use vespera_rules::skills::Skills;

use crate::movement::MoveQueue;
use crate::privacy::PrivateTo;

pub struct CharactersPlugin;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Controls(pub Entity);

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_character)
            .add_observer(despawn_character);
    }
}

fn spawn_character(
    add: On<Add<AuthorizedClient>>,
    mut commands: Commands,
    mut possessions: MessageWriter<ToClients<Possess>>,
) {
    let client = add.entity;
    let character = commands
        .spawn((
            Replicated,
            Character,
            Position::default(),
            ServerMotion::default(),
            MoveQueue::default(),
            Pack::default(),
            Skills::default(),
            PrivateTo(client),
        ))
        .id();
    commands.entity(client).insert(Controls(character));
    possessions.write(ToClients {
        targets: SendTargets::Single(ClientId::Client(client)),
        message: Possess { character },
    });
}

fn despawn_character(
    remove: On<Remove<ConnectedClient>>,
    clients: Query<&Controls>,
    mut commands: Commands,
) {
    if let Ok(controls) = clients.get(remove.entity) {
        commands.entity(controls.0).despawn();
    }
}
