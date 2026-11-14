use std::time::Duration;

use vespera_testkit::TestShard;

#[test]
fn a_new_player_sees_the_landmark_trees_of_the_valley() {
    let mut shard = TestShard::new();
    let player = shard.connect_client();

    shard.run_for(Duration::from_millis(200));

    let landmarks = shard.landmark_count();
    assert!(landmarks > 0, "the valley has no landmark trees");
    assert_eq!(shard.visible_trees(player), landmarks);
}
