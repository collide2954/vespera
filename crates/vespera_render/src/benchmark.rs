use std::time::Duration;

use bevy::diagnostic::DiagnosticsStore;
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PresentMode, WindowMode};
use vespera_client::LocalPlayer;

use crate::{FrameTimes, GpuPassTimes};

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Benchmark {
    pub duration: Duration,
}

pub struct BenchmarkPlugin;

#[derive(Default)]
struct Recording {
    started: bool,
    elapsed: Duration,
    times: FrameTimes,
    passes: GpuPassTimes,
}

impl Plugin for BenchmarkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            prepare_windows.run_if(resource_exists::<Benchmark>),
        )
        .add_systems(Update, record_frames.run_if(resource_exists::<Benchmark>));
    }
}

fn prepare_windows(mut windows: Query<&mut Window>) {
    for mut window in &mut windows {
        window.present_mode = PresentMode::AutoNoVsync;
        window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current);
    }
}

fn record_frames(
    time: Res<Time<Real>>,
    benchmark: Res<Benchmark>,
    players: Query<(), With<LocalPlayer>>,
    diagnostics: Option<Res<DiagnosticsStore>>,
    mut recording: Local<Recording>,
    mut exit: MessageWriter<AppExit>,
) {
    if players.is_empty() {
        return;
    }
    if !recording.started {
        recording.started = true;
        return;
    }
    let delta = time.delta();
    recording.times.record(delta.as_secs_f32());
    recording.elapsed += delta;
    if let Some(diagnostics) = &diagnostics {
        recording.passes.record(diagnostics);
    }
    if recording.elapsed >= benchmark.duration {
        if let Some(summary) = recording.times.summary() {
            info!("benchmark: {summary}");
        }
        if let Some(passes) = recording.passes.summary() {
            info!("benchmark: {passes}");
        }
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use bevy::window::{MonitorSelection, PresentMode, WindowMode};
    use vespera_client::LocalPlayer;

    use super::{Benchmark, BenchmarkPlugin};

    const FRAME: Duration = Duration::from_millis(20);

    fn app(benchmark: Option<Duration>, with_player: bool) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, BenchmarkPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(FRAME));
        if let Some(duration) = benchmark {
            app.insert_resource(Benchmark { duration });
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
    fn without_a_benchmark_the_window_does_not_change() {
        let window = Window::default();

        assert_eq!(
            window_after_start(None),
            Some((window.present_mode, window.mode))
        );
    }
}
