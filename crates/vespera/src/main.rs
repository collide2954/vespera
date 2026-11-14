use std::env;
use std::error::Error;
use std::net::SocketAddr;

use bevy::prelude::*;
use bevy_replicon_renet::RepliconRenetClientPlugin;
use vespera_client::{ClientLogicPlugins, connect};
use vespera_render::PresentationPlugin;

const DEFAULT_SERVER: &str = "127.0.0.1:5000";

fn main() -> AppExit {
    match run() {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("error: {error}");
            AppExit::error()
        }
    }
}

fn run() -> Result<AppExit, Box<dyn Error>> {
    let server: SocketAddr = env::var("VESPERA_SERVER")
        .unwrap_or_else(|_| DEFAULT_SERVER.to_owned())
        .parse()?;
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Vespera".to_owned(),
                ..default()
            }),
            ..default()
        }),
        ClientLogicPlugins,
        RepliconRenetClientPlugin,
        PresentationPlugin,
    ));
    connect(&mut app, server)?;
    Ok(app.run())
}
