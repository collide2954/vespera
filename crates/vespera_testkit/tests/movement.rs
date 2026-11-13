use std::time::Duration;

use bevy::math::{Vec2, Vec3};
use vespera_motion::RUN_SPEED;
use vespera_testkit::{TICK, TestShard};

const MILLIMETER: f32 = 0.001;

fn walk_forward_and_stop(shard: &mut TestShard, player: vespera_testkit::TestClient) {
    shard.set_input(player, Vec2::X, false);
    shard.run_for(Duration::from_secs(1));
    shard.set_input(player, Vec2::ZERO, false);
    shard.run_for(Duration::from_secs(1));
}

#[test]
fn a_player_moves_in_the_input_direction() {
    let mut shard = TestShard::new();
    let player = shard.connect_client();

    walk_forward_and_stop(&mut shard, player);

    let position = shard.server_position(player);
    assert!(position.x > 4.0, "the player moved only {position}");
    assert!(position.z.abs() < MILLIMETER);
}

#[test]
fn the_prediction_agrees_with_the_server_when_the_player_stops() {
    let mut shard = TestShard::new();
    let player = shard.connect_client();

    walk_forward_and_stop(&mut shard, player);

    let error = shard
        .predicted_position(player)
        .distance(shard.server_position(player));
    assert!(
        error < MILLIMETER,
        "the prediction is {error} m from the server"
    );
}

#[test]
fn the_client_predicts_its_movement_before_the_server_confirms_it() {
    let mut shard = TestShard::new();
    let player = shard.connect_client();

    shard.set_input(player, Vec2::X, false);
    shard.run_for(TICK * 2);

    assert!(shard.predicted_position(player).x > 0.0);
    assert_eq!(shard.server_position(player), Vec3::ZERO);
}

#[test]
fn the_client_accepts_a_correction_from_the_server() {
    let mut shard = TestShard::new();
    let player = shard.connect_client();
    shard.run_for(Duration::from_millis(100));

    shard.teleport(player, Vec3::new(10.0, 0.0, 0.0));
    shard.run_for(Duration::from_millis(100));

    let error = shard
        .predicted_position(player)
        .distance(Vec3::new(10.0, 0.0, 0.0));
    assert!(
        error < MILLIMETER,
        "the prediction is {error} m from the server"
    );
}

#[test]
fn other_players_see_the_movement() {
    let mut shard = TestShard::new();
    let mover = shard.connect_client();
    let watcher = shard.connect_client();

    walk_forward_and_stop(&mut shard, mover);

    let error = shard
        .seen_position(watcher, mover)
        .distance(shard.server_position(mover));
    assert!(error < MILLIMETER, "the watcher sees an error of {error} m");
}

#[test]
fn extra_move_commands_do_not_make_a_player_faster() {
    let mut shard = TestShard::new();
    let cheater = shard.connect_client();

    shard.flood_move_commands(cheater, Vec2::X, 5, Duration::from_secs(1));

    assert!(shard.server_position(cheater).x <= RUN_SPEED);
}
