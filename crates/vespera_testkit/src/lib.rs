#![expect(
    clippy::expect_used,
    reason = "the test kit is test code, so a failed expectation stops the test with a clear message"
)]

mod udp;

use std::path::Path;
use std::time::Duration;

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;
use bevy_replicon::prelude::*;
use bevy_replicon::shared::server_entity_map::ServerEntityMap;
use bevy_replicon::test_app::{ServerTestAppExt, TestClientEntity};
use vespera_client::{ClientLogicPlugins, LocalMoveInput, LocalPlayer, Prediction};
use vespera_content::{Content, ItemId, TreeId};
use vespera_core::Position;
use vespera_motion::MoveInput;
use vespera_protocol::{Character, InteractIntent, MoveCommand, ServerMotion};
use vespera_rules::pack::Pack;
use vespera_rules::skills::{Skill, Skills};
use vespera_rules::woodcutting::Tree;
use vespera_server::{Controls, ShardPlugins};

pub use udp::UdpTestShard;

pub const TICK: Duration = Duration::from_nanos(16_666_667);

const MAX_WAIT_TICKS: u32 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestClient(usize);

pub struct TestShard {
    server: App,
    clients: Vec<App>,
}

impl TestShard {
    #[must_use]
    pub fn new() -> Self {
        let mut server = App::new();
        server
            .add_plugins((
                MinimalPlugins,
                StatesPlugin,
                ShardPlugins.set(ServerPlugin::new(PostUpdate)),
            ))
            .insert_resource(repository_content());
        use_fixed_ticks(&mut server);
        server.finish();
        Self {
            server,
            clients: Vec::new(),
        }
    }

    #[must_use]
    pub fn content(&self) -> &Content {
        self.server.world().resource::<Content>()
    }

    pub fn connect_client(&mut self) -> TestClient {
        let mut client = App::new();
        client.add_plugins((MinimalPlugins, StatesPlugin, ClientLogicPlugins));
        use_fixed_ticks(&mut client);
        client.finish();
        self.server.connect_client(&mut client);
        self.clients.push(client);
        TestClient(self.clients.len() - 1)
    }

    pub fn spawn_tree(&mut self, kind: &str, position: Vec3) -> Entity {
        self.server
            .world_mut()
            .spawn((Replicated, Tree(TreeId::new(kind)), Position(position)))
            .id()
    }

    pub fn interact(&mut self, client: TestClient, target: Entity) {
        let local = self.wait_for_entity(client, target);
        self.client_app(client)
            .world_mut()
            .write_message(InteractIntent { target: local });
    }

    pub fn run_for(&mut self, duration: Duration) {
        let mut elapsed = Duration::ZERO;
        while elapsed < duration {
            self.step();
            elapsed += TICK;
        }
    }

    #[must_use]
    pub fn is_connected(&self, client: TestClient) -> bool {
        self.clients.get(client.0).is_some_and(|app| {
            *app.world().resource::<State<ClientState>>() == ClientState::Connected
        })
    }

    pub fn authorized_client_count(&mut self) -> usize {
        count::<With<AuthorizedClient>>(&mut self.server)
    }

    pub fn controls_a_character(&mut self, client: TestClient) -> bool {
        count::<(With<LocalPlayer>, With<Character>)>(self.client_app(client)) == 1
    }

    pub fn pack_count(&mut self, client: TestClient, item: &str) -> u32 {
        self.local_player::<Pack>(client)
            .map_or(0, |pack| pack.count(&ItemId::new(item)))
    }

    pub fn experience(&mut self, client: TestClient, skill: Skill) -> u32 {
        self.local_player::<Skills>(client)
            .map_or(0, |skills| skills.experience(skill))
    }

    pub fn visible_characters(&mut self, client: TestClient) -> usize {
        count::<With<Character>>(self.client_app(client))
    }

    pub fn visible_packs(&mut self, client: TestClient) -> usize {
        count::<With<Pack>>(self.client_app(client))
    }

    pub fn set_input(&mut self, client: TestClient, direction: Vec2, sprint: bool) {
        self.client_app(client)
            .insert_resource(LocalMoveInput(MoveInput { direction, sprint }));
    }

    pub fn server_position(&mut self, client: TestClient) -> Vec3 {
        let character = self.server_character(client);
        self.server
            .world()
            .get::<Position>(character)
            .expect("the character has a position")
            .0
    }

    pub fn predicted_position(&mut self, client: TestClient) -> Vec3 {
        self.local_player::<Prediction>(client)
            .expect("the client predicts its character")
            .position()
    }

    pub fn seen_position(&mut self, observer: TestClient, owner: TestClient) -> Vec3 {
        let character = self.server_character(owner);
        let local = self.wait_for_entity(observer, character);
        self.client_app(observer)
            .world()
            .get::<Position>(local)
            .expect("the observer sees the position")
            .0
    }

    pub fn teleport(&mut self, client: TestClient, position: Vec3) {
        let character = self.server_character(client);
        let mut entity = self.server.world_mut().entity_mut(character);
        entity.insert(Position(position));
        if let Some(mut motion) = entity.get_mut::<ServerMotion>() {
            motion.position = position;
            motion.velocity = Vec3::ZERO;
        }
    }

    pub fn flood_move_commands(
        &mut self,
        client: TestClient,
        direction: Vec2,
        commands_per_tick: u32,
        duration: Duration,
    ) {
        let input = MoveInput {
            direction,
            sprint: false,
        };
        let mut sequence = 1_000_000;
        let mut elapsed = Duration::ZERO;
        while elapsed < duration {
            for _ in 0..commands_per_tick {
                sequence += 1;
                self.client_app(client)
                    .world_mut()
                    .write_message(MoveCommand { sequence, input });
            }
            self.step();
            elapsed += TICK;
        }
    }

    fn step(&mut self) {
        self.server.update();
        for client in &mut self.clients {
            self.server.exchange_with_client(client);
            client.update();
        }
    }

    fn client_app(&mut self, client: TestClient) -> &mut App {
        self.clients
            .get_mut(client.0)
            .expect("the test client belongs to this shard")
    }

    fn server_character(&mut self, client: TestClient) -> Entity {
        let client_entity = **self
            .client_app(client)
            .world()
            .resource::<TestClientEntity>();
        self.server
            .world()
            .get::<Controls>(client_entity)
            .expect("the client controls a character")
            .0
    }

    fn wait_for_entity(&mut self, client: TestClient, server_entity: Entity) -> Entity {
        for _ in 0..MAX_WAIT_TICKS {
            let map = self
                .client_app(client)
                .world()
                .resource::<ServerEntityMap>();
            if let Some(local) = map.to_client().get(&server_entity) {
                return *local;
            }
            self.step();
        }
        panic!("the client did not receive the entity {server_entity}");
    }

    fn local_player<C: Component + Clone>(&mut self, client: TestClient) -> Option<C> {
        let world = self.client_app(client).world_mut();
        world
            .query_filtered::<&C, With<LocalPlayer>>()
            .iter(world)
            .next()
            .cloned()
    }
}

impl Default for TestShard {
    fn default() -> Self {
        Self::new()
    }
}

fn repository_content() -> Content {
    Content::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content"))
        .expect("the repository content is valid")
}

fn use_fixed_ticks(app: &mut App) {
    app.insert_resource(Time::<Fixed>::from_duration(TICK))
        .insert_resource(TimeUpdateStrategy::ManualDuration(TICK));
}

fn count<F: bevy::ecs::query::QueryFilter>(app: &mut App) -> usize {
    let world = app.world_mut();
    world.query_filtered::<(), F>().iter(world).count()
}
