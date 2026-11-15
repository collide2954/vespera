use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};

use bevy::diagnostic::DiagnosticsStore;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::render::diagnostic::RenderDiagnosticsPlugin;
use bevy::render::render_resource::PipelineCache;
use bevy::render::renderer::RenderDevice;
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::window::{MonitorSelection, PresentMode, WindowMode};
use bevy::winit::WinitSettings;
use vespera_client::LocalPlayer;

use crate::report::{Memory, report_json, write_report};
use crate::{FrameSummary, FrameTimes, GpuPassSummary, GpuPassTimes};

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct Benchmark {
    pub duration: Duration,
    pub reports: Option<PathBuf>,
}

pub struct BenchmarkPlugin;

const READY_FRAMES: u32 = 30;

#[derive(Resource, Clone)]
struct PipelinesReady(Arc<AtomicBool>);

impl PipelinesReady {
    fn new(ready: bool) -> Self {
        Self(Arc::new(AtomicBool::new(ready)))
    }

    fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    fn set(&self, ready: bool) {
        self.0.store(ready, Ordering::Relaxed);
    }
}

/// Enables the benchmark and the GPU timers of the render passes.
///
/// The benchmark starts to record frames when no shader pipeline waits for compilation. With the `tracing-tracy` feature, the render plugin of Bevy adds the GPU timers. Thus, this function adds them only when they are not in the app.
pub fn enable_benchmark(app: &mut App, benchmark: Benchmark) {
    let pipelines = PipelinesReady::new(false);
    app.insert_resource(benchmark)
        .insert_resource(pipelines.clone());
    if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
        render_app.insert_resource(pipelines).add_systems(
            Render,
            report_pipeline_readiness.in_set(RenderSystems::Cleanup),
        );
    }
    if !app.is_plugin_added::<RenderDiagnosticsPlugin>() {
        app.add_plugins(RenderDiagnosticsPlugin);
    }
}

fn report_pipeline_readiness(cache: Res<PipelineCache>, pipelines: Res<PipelinesReady>) {
    pipelines.set(cache.waiting_pipelines().next().is_none());
}

#[derive(Default)]
struct Recording {
    ready_frames: u32,
    started: bool,
    finished: bool,
    elapsed: Duration,
    times: FrameTimes,
    passes: GpuPassTimes,
}

impl Plugin for BenchmarkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            prepare_measurement.run_if(resource_exists::<Benchmark>),
        )
        .add_systems(Update, record_frames.run_if(resource_exists::<Benchmark>));
    }
}

fn prepare_measurement(mut commands: Commands, mut windows: Query<&mut Window>) {
    commands.insert_resource(WinitSettings::continuous());
    for mut window in &mut windows {
        window.present_mode = PresentMode::AutoNoVsync;
        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    }
}

#[derive(SystemParam)]
struct Sources<'w> {
    pipelines: Option<Res<'w, PipelinesReady>>,
    diagnostics: Option<Res<'w, DiagnosticsStore>>,
    device: Option<Res<'w, RenderDevice>>,
}

fn record_frames(
    time: Res<Time<Real>>,
    benchmark: Res<Benchmark>,
    players: Query<(), With<LocalPlayer>>,
    sources: Sources,
    mut recording: Local<Recording>,
    mut exit: MessageWriter<AppExit>,
) {
    if recording.finished || players.is_empty() {
        return;
    }
    if !recording.started {
        let ready = match &sources.pipelines {
            Some(pipelines) => pipelines.get(),
            None => true,
        };
        recording.ready_frames = if ready { recording.ready_frames + 1 } else { 0 };
        if recording.ready_frames >= READY_FRAMES {
            recording.started = true;
            info!(
                "benchmark: recording starts after {:.1} s",
                time.elapsed_secs()
            );
        }
        return;
    }
    let delta = time.delta();
    recording.times.record(delta.as_secs_f32());
    recording.elapsed += delta;
    if let Some(diagnostics) = &sources.diagnostics {
        recording.passes.record(diagnostics);
    }
    if recording.elapsed >= benchmark.duration {
        recording.finished = true;
        let frames = recording.times.summary();
        let passes = recording.passes.summary();
        let memory = Memory::measure(sources.device.as_deref());
        if let Some(summary) = &frames {
            info!("benchmark: {summary}");
        }
        if let Some(passes) = &passes {
            info!("benchmark: {passes}");
        }
        info!("benchmark: {memory}");
        if let (Some(folder), Some(summary)) = (&benchmark.reports, &frames) {
            save_report(folder, summary, passes.as_ref(), &memory);
        }
        exit.write(AppExit::Success);
    }
}

fn save_report(
    folder: &Path,
    frames: &FrameSummary,
    passes: Option<&GpuPassSummary>,
    memory: &Memory,
) {
    let saved = report_json(frames, passes, memory)
        .map_err(io::Error::from)
        .and_then(|json| write_report(folder, &json, SystemTime::now()));
    match saved {
        Ok(path) => info!("benchmark: report written to {}", path.display()),
        Err(error) => warn!(
            "benchmark: cannot write the report to {}: {error}",
            folder.display()
        ),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::render::diagnostic::RenderDiagnosticsPlugin;
    use bevy::time::TimeUpdateStrategy;
    use bevy::window::{MonitorSelection, PresentMode, WindowMode};
    use bevy::winit::{UpdateMode, WinitSettings};
    use vespera_client::LocalPlayer;

    use super::{Benchmark, BenchmarkPlugin, PipelinesReady, enable_benchmark};

    const FRAME: Duration = Duration::from_millis(20);

    fn app(benchmark: Option<Duration>, with_player: bool) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, BenchmarkPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME));
        if let Some(duration) = benchmark {
            app.insert_resource(Benchmark {
                duration,
                reports: None,
            });
        }
        if with_player {
            app.world_mut().spawn(LocalPlayer);
        }
        app
    }

    fn run(app: &mut App, frames: usize) -> Option<AppExit> {
        for _ in 0..frames {
            app.update();
            if let Some(exit) = app.should_exit() {
                return Some(exit);
            }
        }
        None
    }

    #[test]
    fn the_benchmark_ends_the_app_after_its_duration() {
        let mut app = app(Some(Duration::from_secs(1)), true);

        assert_eq!(run(&mut app, 100), Some(AppExit::Success));
    }

    #[test]
    fn the_benchmark_does_not_end_the_app_too_early() {
        let mut app = app(Some(Duration::from_secs(1)), true);

        assert_eq!(run(&mut app, 40), None);
    }

    #[test]
    fn the_benchmark_waits_for_the_local_player() {
        let mut app = app(Some(Duration::from_secs(1)), false);

        assert_eq!(run(&mut app, 100), None);
    }

    fn app_with_pipelines(ready: bool) -> App {
        let mut app = app(Some(Duration::from_secs(1)), true);
        app.insert_resource(PipelinesReady::new(ready));
        app
    }

    #[test]
    fn the_benchmark_waits_until_the_shader_pipelines_are_ready() {
        let mut app = app_with_pipelines(false);

        assert_eq!(run(&mut app, 100), None);
    }

    #[test]
    fn the_benchmark_records_when_the_shader_pipelines_are_ready() {
        let mut app = app_with_pipelines(true);

        assert_eq!(run(&mut app, 100), Some(AppExit::Success));
    }

    #[test]
    fn the_benchmark_writes_one_json_report_into_its_folder() {
        let directory = tempfile::tempdir().unwrap();
        let folder = directory.path().join("benchmarks");
        let mut app = app(None, true);
        app.insert_resource(Benchmark {
            duration: Duration::from_secs(1),
            reports: Some(folder.clone()),
        });

        assert_eq!(run(&mut app, 100), Some(AppExit::Success));
        app.update();
        app.update();

        let reports: Vec<_> = std::fs::read_dir(&folder)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(reports.len(), 1);
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&reports[0]).unwrap()).unwrap();
        assert!(json["frames"].as_u64().is_some_and(|frames| frames > 0));
        assert!(json.get("process_memory_mb").is_some());
    }

    #[test]
    fn without_a_benchmark_the_app_keeps_running() {
        let mut app = app(None, true);

        assert_eq!(run(&mut app, 100), None);
    }

    fn window_after_start(benchmark: Option<Duration>) -> Option<(PresentMode, WindowMode)> {
        let mut app = app(benchmark, true);
        let window = app.world_mut().spawn(Window::default()).id();

        app.update();

        app.world()
            .get::<Window>(window)
            .map(|window| (window.present_mode, window.mode))
    }

    #[test]
    fn the_benchmark_turns_off_vsync_to_measure_more_than_the_display_shows() {
        assert_eq!(
            window_after_start(Some(Duration::from_secs(1))).map(|(present, _)| present),
            Some(PresentMode::AutoNoVsync)
        );
    }

    #[test]
    fn the_benchmark_measures_on_the_full_screen() {
        assert_eq!(
            window_after_start(Some(Duration::from_secs(1))).map(|(_, mode)| mode),
            Some(WindowMode::BorderlessFullscreen(MonitorSelection::Current))
        );
    }

    #[test]
    fn the_benchmark_updates_without_pauses_also_without_the_focus() {
        let mut app = app(Some(Duration::from_secs(1)), true);

        app.update();

        let modes = app
            .world()
            .get_resource::<WinitSettings>()
            .map(|settings| (settings.focused_mode, settings.unfocused_mode));
        assert_eq!(
            modes,
            Some((UpdateMode::Continuous, UpdateMode::Continuous))
        );
    }

    #[test]
    fn the_benchmark_measures_the_render_passes_on_the_gpu() {
        let mut app = App::new();

        enable_benchmark(
            &mut app,
            Benchmark {
                duration: Duration::from_secs(1),
                reports: None,
            },
        );

        assert!(app.is_plugin_added::<RenderDiagnosticsPlugin>());
        assert!(app.world().contains_resource::<Benchmark>());
        assert!(app.world().contains_resource::<PipelinesReady>());
    }

    #[test]
    fn the_benchmark_keeps_the_gpu_timers_that_the_render_plugin_added() {
        let mut app = App::new();
        app.add_plugins(RenderDiagnosticsPlugin);

        enable_benchmark(
            &mut app,
            Benchmark {
                duration: Duration::from_secs(1),
                reports: None,
            },
        );

        assert!(app.world().contains_resource::<Benchmark>());
    }

    #[test]
    fn without_a_benchmark_the_window_does_not_change() {
        let window = Window::default();

        assert_eq!(
            window_after_start(None),
            Some((window.present_mode, window.mode))
        );
    }
}
