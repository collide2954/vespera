use std::time::Duration;

use bevy::math::Vec3;
use vespera_content::TreeId;
use vespera_rules::skills::Skill;
use vespera_testkit::TestShard;

#[test]
fn player_receives_oak_logs_after_cutting_an_oak_tree() {
    let mut shard = TestShard::new();
    let oak = shard.spawn_tree("oak", Vec3::new(1.5, 0.0, 0.0));
    let player = shard.connect_client();
    let chop_time = shard
        .content()
        .tree(&TreeId::new("oak"))
        .map(|oak| Duration::from_secs_f32(oak.seconds_per_log))
        .unwrap();

    shard.interact(player, oak);
    shard.run_for(chop_time + Duration::from_millis(500));

    assert_eq!(shard.pack_count(player, "oak_logs"), 1);
    assert!(shard.experience(player, Skill::Woodcutting) > 0);
}
