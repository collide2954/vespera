use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_replicon_renet::RepliconRenetServerPlugin;
use vespera_content::Content;
use vespera_motion::SIMULATION_HZ;
use vespera_server::{ShardPlugins, listen};

const DEFAULT_ADDRESS: &str = "127.0.0.1:5000";
const DEFAULT_CONTENT_DIRECTORY: &str = "content";

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
    let address: SocketAddr = env::var("VESPERA_SHARD_ADDRESS")
        .unwrap_or_else(|_| DEFAULT_ADDRESS.to_owned())
        .parse()?;
    let content_directory = env::var_os("VESPERA_CONTENT")
        .map_or_else(|| PathBuf::from(DEFAULT_CONTENT_DIRECTORY), PathBuf::from);
    let content = Content::load(&content_directory)?;
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
    .insert_resource(content);
    let local_address = listen(&mut app, address)?;
    info!("the shard listens on {local_address}");
    Ok(app.run())
}
