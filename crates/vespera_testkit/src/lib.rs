use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_replicon::prelude::*;
use bevy_replicon::test_app::ServerTestAppExt;
use vespera_client::ClientLogicPlugins;
use vespera_server::ShardPlugins;

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
        server.add_plugins((
            MinimalPlugins,
            StatesPlugin,
            ShardPlugins.set(ServerPlugin::new(PostUpdate)),
        ));
        server.finish();
        Self {
            server,
            clients: Vec::new(),
        }
    }

    pub fn connect_client(&mut self) -> TestClient {
        let mut client = App::new();
        client.add_plugins((MinimalPlugins, StatesPlugin, ClientLogicPlugins));
        client.finish();
        self.server.connect_client(&mut client);
        self.clients.push(client);
        TestClient(self.clients.len() - 1)
    }

    #[must_use]
    pub fn is_connected(&self, client: TestClient) -> bool {
        self.clients.get(client.0).is_some_and(|app| {
            *app.world().resource::<State<ClientState>>() == ClientState::Connected
        })
    }

    pub fn authorized_client_count(&mut self) -> usize {
        self.server
            .world_mut()
            .query_filtered::<(), With<AuthorizedClient>>()
            .iter(self.server.world())
            .count()
    }
}

impl Default for TestShard {
    fn default() -> Self {
        Self::new()
    }
}
