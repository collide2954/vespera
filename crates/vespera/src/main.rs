use std::env;
use std::net::SocketAddr;

use anyhow::{Context, Result};
use bevy::DefaultPlugins;
use bevy::app::{App, AppExit, PluginGroup};
use bevy::utils::default;
use bevy::window::{Window, WindowPlugin};
use bevy_replicon_renet::RepliconRenetClientPlugin;
use vespera_client::{ClientLogicPlugins, connect};
use vespera_render::PresentationPlugin;

const DEFAULT_SERVER: &str = "127.0.0.1:5000";

fn main() -> AppExit {
    match run() {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("error: {error:#}");
            AppExit::error()
        }
    }
}

fn run() -> Result<AppExit> {
    let server: SocketAddr = env::var("VESPERA_SERVER")
        .unwrap_or_else(|_| DEFAULT_SERVER.to_owned())
        .parse()
        .context("VESPERA_SERVER is not a socket address")?;
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
