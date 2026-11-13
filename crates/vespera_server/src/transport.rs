use std::net::{SocketAddr, UdpSocket};
use std::time::{SystemTime, SystemTimeError};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_replicon_renet::renet::ConnectionConfig;
use bevy_replicon_renet::{RenetChannelsExt, RenetServer};
use vespera_protocol::PROTOCOL_ID;

pub const MAX_PLAYERS: usize = 1000;

#[derive(Debug, thiserror::Error)]
pub enum ListenError {
    #[error("cannot open the UDP socket")]
    Socket(#[from] std::io::Error),
    #[error("the system clock is before 1970")]
    Clock(#[from] SystemTimeError),
}

pub fn listen(app: &mut App, address: SocketAddr) -> Result<SocketAddr, ListenError> {
    let channels = app.world().resource::<RepliconChannels>();
    let server = RenetServer::new(ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..Default::default()
    });
    let socket = UdpSocket::bind(address)?;
    let local_address = socket.local_addr()?;
    let config = ServerConfig {
        current_time: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?,
        max_clients: MAX_PLAYERS,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![local_address],
        authentication: ServerAuthentication::Unsecure,
    };
    let transport = NetcodeServerTransport::new(config, socket)?;
    app.insert_resource(server).insert_resource(transport);
    Ok(local_address)
}
