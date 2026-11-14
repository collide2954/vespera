use std::time::Duration;

use bevy::prelude::*;
use vespera_client::LocalPlayer;

use crate::FrameTimes;

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
}

impl Plugin for BenchmarkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, record_frames.run_if(resource_exists::<Benchmark>));
    }
}

fn record_frames(
    time: Res<Time<Real>>,
    benchmark: Res<Benchmark>,
    players: Query<(), With<LocalPlayer>>,
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
    if recording.elapsed >= benchmark.duration {
        if let Some(summary) = recording.times.summary() {
            info!("benchmark: {summary}");
        }
        exit.write(AppExit::Success);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
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
}
