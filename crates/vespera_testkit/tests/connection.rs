use std::time::Duration;

use vespera_testkit::TestShard;

#[test]
fn a_client_connects_to_the_shard() {
    let mut shard = TestShard::new();

    let client = shard.connect_client();

    assert!(shard.is_connected(client));
    assert_eq!(shard.authorized_client_count(), 1);
}

#[test]
fn two_clients_connect_to_the_same_shard() {
    let mut shard = TestShard::new();

    let first = shard.connect_client();
    let second = shard.connect_client();

    assert!(shard.is_connected(first));
    assert!(shard.is_connected(second));
    assert_eq!(shard.authorized_client_count(), 2);
}

#[test]
fn a_connected_player_controls_one_character() {
    let mut shard = TestShard::new();
    let player = shard.connect_client();

    shard.run_for(Duration::from_millis(100));

    assert!(shard.controls_a_character(player));
}
