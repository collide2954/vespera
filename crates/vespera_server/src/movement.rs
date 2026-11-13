use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use vespera_core::Position;
use vespera_motion::{MotionState, step};
use vespera_protocol::{MoveCommand, ServerMotion};

use crate::characters::Controls;

pub struct MovementPlugin;

const MAX_QUEUED_COMMANDS: usize = 8;

#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct MoveQueue(VecDeque<MoveCommand>);

impl Plugin for MovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            queue_commands
                .after(ServerSystems::Receive)
                .run_if(in_state(ServerState::Running)),
        )
        .add_systems(FixedUpdate, apply_commands);
    }
}

fn queue_commands(
    mut received: MessageReader<FromClient<MoveCommand>>,
    clients: Query<&Controls>,
    mut queues: Query<&mut MoveQueue>,
) {
    for command in received.read() {
        let ClientId::Client(client) = command.client_id else {
            continue;
        };
        let Some(mut queue) = clients
            .get(client)
            .ok()
            .and_then(|controls| queues.get_mut(controls.0).ok())
        else {
            continue;
        };
        if queue.0.len() == MAX_QUEUED_COMMANDS {
            queue.0.pop_front();
        }
        queue.0.push_back(command.message);
    }
}

fn apply_commands(
    time: Res<Time>,
    mut characters: Query<(&mut MoveQueue, &mut ServerMotion, &mut Position)>,
) {
    for (mut queue, mut motion, mut position) in &mut characters {
        let Some(command) = next_fresh_command(&mut queue, motion.sequence) else {
            continue;
        };
        let start = MotionState {
            position: position.0,
            velocity: motion.velocity,
        };
        let state = step(start, command.input, time.delta_secs());
        position.set_if_neq(Position(state.position));
        *motion = ServerMotion {
            sequence: command.sequence,
            position: state.position,
            velocity: state.velocity,
        };
    }
}

fn next_fresh_command(queue: &mut MoveQueue, last_sequence: u32) -> Option<MoveCommand> {
    while let Some(command) = queue.0.pop_front() {
        if command.sequence > last_sequence {
            return Some(command);
        }
    }
    None
}
