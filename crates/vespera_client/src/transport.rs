use std::hash::{BuildHasher, RandomState};
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{SystemTime, SystemTimeError};

use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::netcode::{ClientAuthentication, NetcodeClientTransport, NetcodeError};
use bevy_replicon_renet::renet::ConnectionConfig;
use bevy_replicon_renet::{RenetChannelsExt, RenetClient};
use vespera_protocol::PROTOCOL_ID;

#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("cannot open a UDP socket")]
    Socket(#[from] std::io::Error),
    #[error("the system clock is before 1970")]
    Clock(#[from] SystemTimeError),
    #[error("cannot start the netcode client")]
    Netcode(#[from] Box<NetcodeError>),
}

pub fn connect(app: &mut App, server_address: SocketAddr) -> Result<(), ConnectError> {
    let channels = app.world().resource::<RepliconChannels>();
    let client = RenetClient::new(ConnectionConfig {
        server_channels_config: channels.server_configs(),
        client_channels_config: channels.client_configs(),
        ..Default::default()
    });
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?;
    let authentication = ClientAuthentication::Unsecure {
        protocol_id: PROTOCOL_ID,
        client_id: RandomState::new().hash_one(current_time),
        server_addr: server_address,
        user_data: None,
    };
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    let transport =
        NetcodeClientTransport::new(current_time, authentication, socket).map_err(Box::new)?;
    app.insert_resource(client).insert_resource(transport);
    Ok(())
}
