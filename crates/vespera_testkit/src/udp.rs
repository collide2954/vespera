use std::net::{Ipv4Addr, SocketAddr};
use std::thread;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_replicon_renet::{RepliconRenetClientPlugin, RepliconRenetServerPlugin};
use vespera_client::{ClientLogicPlugins, LocalPlayer};
use vespera_protocol::Character;
use vespera_server::ShardPlugins;

use crate::repository_content;

const POLL_INTERVAL: Duration = Duration::from_millis(2);

pub struct UdpTestShard {
    server: App,
    client: App,
}

impl UdpTestShard {
    #[must_use]
    pub fn new() -> Self {
        let mut server = App::new();
        server
            .add_plugins((
                MinimalPlugins,
                StatesPlugin,
                ShardPlugins,
                RepliconRenetServerPlugin,
            ))
            .insert_resource(repository_content());
        server.finish();
        let address =
            vespera_server::listen(&mut server, SocketAddr::from((Ipv4Addr::LOCALHOST, 0)))
                .expect("the shard listens on a free local port");
        let mut client = App::new();
        client.add_plugins((
            MinimalPlugins,
            StatesPlugin,
            ClientLogicPlugins,
            RepliconRenetClientPlugin,
        ));
        client.finish();
        vespera_client::connect(&mut client, address).expect("the client starts to connect");
        Self { server, client }
    }

    pub fn wait_until(&mut self, timeout: Duration, condition: fn(&mut Self) -> bool) -> bool {
        let start = Instant::now();
        while start.elapsed() < timeout {
            self.server.update();
            self.client.update();
            if condition(self) {
                return true;
            }
            thread::sleep(POLL_INTERVAL);
        }
        false
    }

    pub fn controls_a_character(&mut self) -> bool {
        let world = self.client.world_mut();
        world
            .query_filtered::<(), (With<LocalPlayer>, With<Character>)>()
            .iter(world)
            .count()
            == 1
    }
}

impl Default for UdpTestShard {
    fn default() -> Self {
        Self::new()
    }
}
