use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use vespera_content::{Content, TreeDefinition, TreeId};
use vespera_core::Position;

use crate::pack::Pack;
use crate::skills::{Skill, Skills};

pub const REACH: f32 = 2.5;

pub struct WoodcuttingPlugin;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChopRequest {
    pub woodcutter: Entity,
    pub tree: Entity,
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tree(pub TreeId);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Felled;

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Chopping {
    tree: Entity,
    seconds: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
struct LogsLeft(u8);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
struct Regrowth {
    seconds: f32,
}

impl Plugin for WoodcuttingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ChopRequest>().add_systems(
            FixedUpdate,
            (prepare_trees, start_chopping, chop, regrow).chain(),
        );
    }
}

fn prepare_trees(
    mut commands: Commands,
    content: Res<Content>,
    trees: Query<(Entity, &Tree), (Without<LogsLeft>, Without<Felled>)>,
) {
    for (entity, tree) in &trees {
        if let Some(definition) = content.tree(&tree.0) {
            commands.entity(entity).insert(LogsLeft(definition.logs));
        }
    }
}

fn start_chopping(
    mut requests: MessageReader<ChopRequest>,
    mut commands: Commands,
    content: Res<Content>,
    woodcutters: Query<(&Position, &Skills)>,
    trees: Query<(&Tree, &Position), Without<Felled>>,
) {
    for request in requests.read() {
        let (Ok((position, skills)), Ok((tree, tree_position))) =
            (woodcutters.get(request.woodcutter), trees.get(request.tree))
        else {
            continue;
        };
        let Some(definition) = content.tree(&tree.0) else {
            continue;
        };
        if in_reach(position, tree_position) && skills.level(Skill::Woodcutting) >= definition.level
        {
            commands.entity(request.woodcutter).insert(Chopping {
                tree: request.tree,
                seconds: 0.0,
            });
        }
    }
}

fn in_reach(woodcutter: &Position, tree: &Position) -> bool {
    woodcutter.0.distance(tree.0) <= REACH
}

fn chop(
    time: Res<Time>,
    content: Res<Content>,
    mut commands: Commands,
    mut woodcutters: Query<(Entity, &Position, &mut Chopping, &mut Pack, &mut Skills)>,
    mut trees: Query<(&Tree, &Position, &mut LogsLeft), Without<Felled>>,
) {
    for (woodcutter, position, mut chopping, mut pack, mut skills) in &mut woodcutters {
        let tree = chopping.tree;
        let Ok((kind, tree_position, mut logs_left)) = trees.get_mut(tree) else {
            commands.entity(woodcutter).remove::<Chopping>();
            continue;
        };
        let can_chop = in_reach(position, tree_position) && logs_left.0 > 0;
        let Some(definition) = content.tree(&kind.0).filter(|_| can_chop) else {
            commands.entity(woodcutter).remove::<Chopping>();
            continue;
        };
        chopping.seconds += time.delta_secs();
        if chopping.seconds < definition.seconds_per_log {
            continue;
        }
        chopping.seconds -= definition.seconds_per_log;
        if !give_log(&content, definition, &mut pack, &mut skills) {
            commands.entity(woodcutter).remove::<Chopping>();
            continue;
        }
        logs_left.0 -= 1;
        if logs_left.0 == 0 {
            commands.entity(tree).insert((
                Felled,
                Regrowth {
                    seconds: definition.regrow_seconds,
                },
            ));
            commands.entity(woodcutter).remove::<Chopping>();
        }
    }
}

fn give_log(
    content: &Content,
    definition: &TreeDefinition,
    pack: &mut Pack,
    skills: &mut Skills,
) -> bool {
    let stack_size = content
        .item(&definition.log)
        .map_or(1, |item| item.stack_size);
    if pack.add(&definition.log, 1, stack_size).is_err() {
        return false;
    }
    skills.add_experience(Skill::Woodcutting, definition.experience);
    true
}

fn regrow(
    time: Res<Time>,
    content: Res<Content>,
    mut commands: Commands,
    mut trees: Query<(Entity, &Tree, &mut Regrowth)>,
) {
    for (entity, tree, mut regrowth) in &mut trees {
        regrowth.seconds -= time.delta_secs();
        if regrowth.seconds <= 0.0 {
            let logs = content
                .tree(&tree.0)
                .map_or(1, |definition| definition.logs);
            commands
                .entity(entity)
                .remove::<(Felled, Regrowth)>()
                .insert(LogsLeft(logs));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use vespera_content::{Content, ItemId, TreeId};
    use vespera_core::Position;

    use super::{ChopRequest, Chopping, Felled, Tree, WoodcuttingPlugin};
    use crate::pack::Pack;
    use crate::skills::{Skill, Skills};

    const TICK: Duration = Duration::from_nanos(16_666_667);

    fn content(level: u8) -> Content {
        Content::from_sources(
            "{ \"oak_logs\": (stack_size: 1), \"stones\": (stack_size: 1) }",
            &format!(
                "{{ \"oak\": (level: {level}, log: \"oak_logs\", experience: 25, seconds_per_log: 3.0, logs: 3, regrow_seconds: 30.0) }}"
            ),
        )
        .unwrap()
    }

    struct Scene {
        app: App,
        woodcutter: Entity,
        tree: Entity,
    }

    impl Scene {
        fn new(tree_level: u8, woodcutter_at: Vec3) -> Self {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, WoodcuttingPlugin))
                .insert_resource(content(tree_level))
                .insert_resource(Time::<Fixed>::from_duration(TICK))
                .insert_resource(TimeUpdateStrategy::ManualDuration(TICK));
            let woodcutter = app
                .world_mut()
                .spawn((Position(woodcutter_at), Pack::default(), Skills::default()))
                .id();
            let tree = app
                .world_mut()
                .spawn((Tree(TreeId::new("oak")), Position(Vec3::X)))
                .id();
            app.update();
            Self {
                app,
                woodcutter,
                tree,
            }
        }

        fn chop(&mut self) {
            self.app.world_mut().write_message(ChopRequest {
                woodcutter: self.woodcutter,
                tree: self.tree,
            });
        }

        fn run_for(&mut self, seconds: f32) {
            let target = Duration::from_secs_f32(seconds);
            let mut elapsed = Duration::ZERO;
            while elapsed < target {
                self.app.update();
                elapsed += TICK;
            }
        }

        fn logs(&self) -> u32 {
            self.app
                .world()
                .get::<Pack>(self.woodcutter)
                .unwrap()
                .count(&ItemId::new("oak_logs"))
        }

        fn experience(&self) -> u32 {
            self.app
                .world()
                .get::<Skills>(self.woodcutter)
                .unwrap()
                .experience(Skill::Woodcutting)
        }

        fn is_felled(&self) -> bool {
            self.app.world().get::<Felled>(self.tree).is_some()
        }

        fn is_chopping(&self) -> bool {
            self.app.world().get::<Chopping>(self.woodcutter).is_some()
        }
    }

    #[test]
    fn a_woodcutter_receives_a_log_and_experience_after_the_chop_time() {
        let mut scene = Scene::new(1, Vec3::ZERO);
        scene.chop();

        scene.run_for(2.9);
        assert_eq!(scene.logs(), 0);

        scene.run_for(0.2);
        assert_eq!(scene.logs(), 1);
        assert_eq!(scene.experience(), 25);
    }

    #[test]
    fn a_tree_falls_after_its_last_log() {
        let mut scene = Scene::new(1, Vec3::ZERO);
        scene.chop();

        scene.run_for(9.1);

        assert_eq!(scene.logs(), 3);
        assert!(scene.is_felled());
        assert!(!scene.is_chopping());
    }

    #[test]
    fn a_felled_tree_grows_again_after_its_regrow_time() {
        let mut scene = Scene::new(1, Vec3::ZERO);
        scene.chop();
        scene.run_for(9.1);

        scene.run_for(30.1);
        assert!(!scene.is_felled());

        scene.chop();
        scene.run_for(3.1);
        assert_eq!(scene.logs(), 4);
    }

    #[test]
    fn a_felled_tree_cannot_be_chopped() {
        let mut scene = Scene::new(1, Vec3::ZERO);
        scene.chop();
        scene.run_for(9.1);

        scene.chop();
        scene.run_for(3.1);

        assert_eq!(scene.logs(), 3);
    }

    #[test]
    fn a_woodcutter_out_of_reach_does_not_chop() {
        let mut scene = Scene::new(1, Vec3::new(10.0, 0.0, 0.0));
        scene.chop();

        scene.run_for(3.1);

        assert_eq!(scene.logs(), 0);
        assert!(!scene.is_chopping());
    }

    #[test]
    fn a_woodcutter_who_walks_out_of_reach_stops_chopping() {
        let mut scene = Scene::new(1, Vec3::ZERO);
        scene.chop();
        scene.run_for(1.0);

        scene
            .app
            .world_mut()
            .get_mut::<Position>(scene.woodcutter)
            .unwrap()
            .0 = Vec3::new(10.0, 0.0, 0.0);
        scene.run_for(3.1);

        assert_eq!(scene.logs(), 0);
        assert!(!scene.is_chopping());
    }

    #[test]
    fn a_woodcutter_below_the_tree_level_does_not_chop() {
        let mut scene = Scene::new(15, Vec3::ZERO);
        scene.chop();

        scene.run_for(3.1);

        assert_eq!(scene.logs(), 0);
    }

    #[test]
    fn a_full_pack_stops_the_chopping() {
        let mut scene = Scene::new(1, Vec3::ZERO);
        scene
            .app
            .world_mut()
            .get_mut::<Pack>(scene.woodcutter)
            .unwrap()
            .add(&ItemId::new("stones"), 28, 1)
            .unwrap();
        scene.chop();

        scene.run_for(3.1);

        assert_eq!(scene.logs(), 0);
        assert!(!scene.is_chopping());
    }

    #[test]
    fn two_woodcutters_cannot_take_more_logs_than_the_tree_has() {
        let mut scene = Scene::new(1, Vec3::ZERO);
        let second = scene
            .app
            .world_mut()
            .spawn((Position(Vec3::ZERO), Pack::default(), Skills::default()))
            .id();
        scene.chop();
        scene.app.world_mut().write_message(ChopRequest {
            woodcutter: second,
            tree: scene.tree,
        });

        scene.run_for(12.1);

        let second_logs = scene
            .app
            .world()
            .get::<Pack>(second)
            .unwrap()
            .count(&ItemId::new("oak_logs"));
        assert_eq!(scene.logs() + second_logs, 3);
    }
}
