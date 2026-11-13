use std::time::Duration;

use vespera_testkit::UdpTestShard;

#[test]
fn a_client_connects_to_a_shard_over_udp_and_controls_a_character() {
    let mut shard = UdpTestShard::new();

    let connected = shard.wait_until(Duration::from_secs(10), UdpTestShard::controls_a_character);

    assert!(
        connected,
        "the client did not receive its character in time"
    );
}
