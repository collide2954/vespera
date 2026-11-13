use std::time::Duration;

use vespera_testkit::TestShard;

#[test]
fn a_player_sees_other_characters_but_only_its_own_pack() {
    let mut shard = TestShard::new();
    let first = shard.connect_client();
    let second = shard.connect_client();

    shard.run_for(Duration::from_millis(100));

    assert_eq!(shard.visible_characters(first), 2);
    assert_eq!(shard.visible_packs(first), 1);
    assert_eq!(shard.visible_characters(second), 2);
    assert_eq!(shard.visible_packs(second), 1);
}
