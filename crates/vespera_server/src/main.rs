use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use bevy::MinimalPlugins;
use bevy::app::{App, AppExit, PluginGroup, ScheduleRunnerPlugin};
use bevy::log::{LogPlugin, info};
use bevy::state::app::StatesPlugin;
use bevy_replicon_renet::RepliconRenetServerPlugin;
use vespera_content::{Content, Landmarks};
use vespera_motion::SIMULATION_HZ;
use vespera_server::{ShardPlugins, listen};

const DEFAULT_ADDRESS: &str = "127.0.0.1:5000";
const DEFAULT_CONTENT_DIRECTORY: &str = "content";

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
    let address: SocketAddr = env::var("VESPERA_SHARD_ADDRESS")
        .unwrap_or_else(|_| DEFAULT_ADDRESS.to_owned())
        .parse()
        .context("VESPERA_SHARD_ADDRESS is not a socket address")?;
    let content_directory = env::var_os("VESPERA_CONTENT")
        .map_or_else(|| PathBuf::from(DEFAULT_CONTENT_DIRECTORY), PathBuf::from);
    let content = Content::load(&content_directory)?;
    let landmarks = Landmarks::load(&content_directory, &content)?;
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(
            1.0 / SIMULATION_HZ,
        ))),
        StatesPlugin,
        LogPlugin::default(),
        ShardPlugins,
        RepliconRenetServerPlugin,
    ))
    .insert_resource(content)
    .insert_resource(landmarks);
    let local_address = listen(&mut app, address)?;
    info!("the shard listens on {local_address}");
    Ok(app.run())
}
