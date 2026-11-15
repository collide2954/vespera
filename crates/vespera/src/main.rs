use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use anyhow::{Context, Result};
use bevy::DefaultPlugins;
use bevy::app::{App, AppExit, PluginGroup};
use bevy::render::RenderPlugin;
use bevy::render::settings::{Backends, Dx12Compiler, WgpuSettings};
use bevy::utils::default;
use bevy::window::{Window, WindowPlugin};
use bevy_replicon_renet::RepliconRenetClientPlugin;
use vespera_client::{ClientLogicPlugins, connect};
use vespera_render::{
    Benchmark, BenchmarkPlugin, DXC_LIBRARY, PresentationPlugin, dxc_next_to, enable_benchmark,
    render_backends, shader_compiler,
};

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
    let benchmark = benchmark_from_environment()?;
    let dxc_library = env::current_exe()
        .ok()
        .and_then(|executable| dxc_next_to(&executable));
    if cfg!(windows) && dxc_library.is_none() {
        eprintln!(
            "warning: {DXC_LIBRARY} is not next to the game, thus Direct3D 12 uses the slow FXC shader compiler. Run cargo xtask fetch-dxc."
        );
    }
    let mut app = App::new();
    app.add_plugins((
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Vespera".to_owned(),
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: Some(render_backends(Backends::from_env(), cfg!(windows))),
                    dx12_shader_compiler: shader_compiler(Dx12Compiler::from_env(), dxc_library),
                    ..default()
                }
                .into(),
                ..default()
            }),
        ClientLogicPlugins,
        RepliconRenetClientPlugin,
        PresentationPlugin,
        BenchmarkPlugin,
    ));
    if let Some(benchmark) = benchmark {
        enable_benchmark(&mut app, benchmark);
    }
    connect(&mut app, server)?;
    Ok(app.run())
}

fn benchmark_from_environment() -> Result<Option<Benchmark>> {
    let Ok(text) = env::var("VESPERA_BENCHMARK_SECONDS") else {
        return Ok(None);
    };
    let seconds: u64 = text
        .parse()
        .context("VESPERA_BENCHMARK_SECONDS is not a whole number of seconds")?;
    Ok(Some(Benchmark {
        duration: Duration::from_secs(seconds),
    }))
}
