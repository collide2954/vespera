use std::collections::VecDeque;

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use vespera_motion::{MotionState, MoveInput, step};
use vespera_protocol::{MoveCommand, ServerMotion};

use crate::LocalPlayer;

const MAX_HISTORY: usize = 240;

pub struct PredictionPlugin;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub struct LocalMoveInput(pub MoveInput);

#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct Prediction {
    state: MotionState,
    sequence: u32,
    history: VecDeque<MoveCommand>,
}

impl Prediction {
    #[must_use]
    pub fn position(&self) -> Vec3 {
        self.state.position
    }
}

impl Plugin for PredictionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalMoveInput>()
            .add_observer(start_prediction)
            .add_systems(
                PreUpdate,
                reconcile
                    .after(ClientSystems::Receive)
                    .run_if(in_state(ClientState::Connected)),
            )
            .add_systems(
                FixedUpdate,
                predict.run_if(in_state(ClientState::Connected)),
            );
    }
}

fn start_prediction(
    add: On<Add<LocalPlayer>>,
    motions: Query<&ServerMotion>,
    mut commands: Commands,
) {
    let state = motions
        .get(add.entity)
        .map_or_else(|_| MotionState::default(), motion_state);
    commands.entity(add.entity).insert(Prediction {
        state,
        ..Prediction::default()
    });
}

fn predict(
    time: Res<Time>,
    input: Res<LocalMoveInput>,
    mut players: Query<&mut Prediction, With<LocalPlayer>>,
    mut sent: MessageWriter<MoveCommand>,
) {
    for mut prediction in &mut players {
        prediction.sequence += 1;
        let command = MoveCommand {
            sequence: prediction.sequence,
            input: input.0,
        };
        prediction.state = step(prediction.state, command.input, time.delta_secs());
        prediction.history.push_back(command);
        if prediction.history.len() > MAX_HISTORY {
            prediction.history.pop_front();
        }
        sent.write(command);
    }
}

fn reconcile(
    time: Res<Time<Fixed>>,
    mut players: Query<
        (&mut Prediction, &ServerMotion),
        (With<LocalPlayer>, Changed<ServerMotion>),
    >,
) {
    let seconds = time.timestep().as_secs_f32();
    for (mut prediction, server) in &mut players {
        prediction
            .history
            .retain(|command| command.sequence > server.sequence);
        let state = prediction
            .history
            .iter()
            .fold(motion_state(server), |state, command| {
                step(state, command.input, seconds)
            });
        prediction.state = state;
    }
}

fn motion_state(motion: &ServerMotion) -> MotionState {
    MotionState {
        position: motion.position,
        velocity: motion.velocity,
    }
}
