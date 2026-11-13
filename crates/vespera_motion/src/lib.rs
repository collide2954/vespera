use bevy::math::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

pub const RUN_SPEED: f32 = 5.4;
pub const SPRINT_SPEED: f32 = 7.2;
pub const ACCELERATION: f32 = 14.4;
pub const DECELERATION: f32 = 20.0;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct MoveInput {
    pub direction: Vec2,
    pub sprint: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct MotionState {
    pub position: Vec3,
    pub velocity: Vec3,
}

#[must_use]
pub fn step(state: MotionState, input: MoveInput, seconds: f32) -> MotionState {
    let direction = if input.direction.is_finite() {
        input.direction.clamp_length_max(1.0)
    } else {
        Vec2::ZERO
    };
    let speed = if input.sprint {
        SPRINT_SPEED
    } else {
        RUN_SPEED
    };
    let target = Vec3::new(direction.x, 0.0, direction.y) * speed;
    let current = Vec3::new(state.velocity.x, 0.0, state.velocity.z);
    let rate = if target.length_squared() > current.length_squared() {
        ACCELERATION
    } else {
        DECELERATION
    };
    let velocity = move_towards(current, target, rate * seconds);
    MotionState {
        position: state.position + velocity * seconds,
        velocity,
    }
}

fn move_towards(current: Vec3, target: Vec3, max_change: f32) -> Vec3 {
    let change = target - current;
    let length = change.length();
    if length <= max_change {
        target
    } else {
        current + change * (max_change / length)
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::{Vec2, Vec3};

    use super::{MotionState, MoveInput, RUN_SPEED, SPRINT_SPEED, step};

    const TICK: f32 = 1.0 / 60.0;

    fn run(state: MotionState, input: MoveInput, seconds: f32) -> MotionState {
        let mut state = state;
        let mut elapsed = 0.0;
        while elapsed < seconds - 1e-6 {
            state = step(state, input, TICK);
            elapsed += TICK;
        }
        state
    }

    fn forward(sprint: bool) -> MoveInput {
        MoveInput {
            direction: Vec2::X,
            sprint,
        }
    }

    #[test]
    fn a_character_without_input_stays_in_place() {
        let state = run(MotionState::default(), MoveInput::default(), 1.0);

        assert_eq!(state, MotionState::default());
    }

    #[test]
    fn a_character_reaches_run_speed_in_less_than_half_a_second() {
        let state = run(MotionState::default(), forward(false), 0.5);

        assert!((state.velocity.length() - RUN_SPEED).abs() < 1e-4);
    }

    #[test]
    fn a_character_moves_in_the_input_direction_on_the_ground() {
        let state = run(MotionState::default(), forward(false), 1.0);

        assert!(state.position.x > 4.0 && state.position.x < RUN_SPEED);
        assert!(state.position.y.abs() < 1e-6);
        assert!(state.position.z.abs() < 1e-6);
    }

    #[test]
    fn sprinting_is_faster_than_running() {
        let running = run(MotionState::default(), forward(false), 1.0);
        let sprinting = run(MotionState::default(), forward(true), 1.0);

        assert!(sprinting.position.x > running.position.x);
        assert!((sprinting.velocity.length() - SPRINT_SPEED).abs() < 1e-4);
    }

    #[test]
    fn a_character_stops_when_the_input_stops() {
        let moving = run(MotionState::default(), forward(false), 1.0);

        let stopped = run(moving, MoveInput::default(), 0.5);

        assert_eq!(stopped.velocity, Vec3::ZERO);
        assert!(stopped.position.x > moving.position.x);
    }

    #[test]
    fn input_longer_than_one_is_shortened() {
        let long = MoveInput {
            direction: Vec2::new(10.0, 0.0),
            sprint: false,
        };

        assert_eq!(
            run(MotionState::default(), long, 1.0),
            run(MotionState::default(), forward(false), 1.0)
        );
    }

    #[test]
    fn input_that_is_not_finite_is_ignored() {
        let broken = MoveInput {
            direction: Vec2::new(f32::NAN, f32::INFINITY),
            sprint: false,
        };

        assert_eq!(
            run(MotionState::default(), broken, 1.0),
            MotionState::default()
        );
    }

    #[test]
    fn the_same_inputs_always_give_the_same_state() {
        let diagonal = MoveInput {
            direction: Vec2::new(0.6, -0.8),
            sprint: true,
        };

        assert_eq!(
            run(MotionState::default(), diagonal, 2.0),
            run(MotionState::default(), diagonal, 2.0)
        );
    }
}
