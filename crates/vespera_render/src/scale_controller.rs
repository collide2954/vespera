use bevy::prelude::*;
use bevy::window::{Monitor, OnMonitor, PresentMode, PrimaryMonitor, PrimaryWindow};

use crate::RenderScale;

const TARGET_FRAME_TIME: f64 = 1.0 / 60.0;
const LEVELS: u8 = 5;
const WINDOW_FRAMES: u16 = 30;
const OVERLOADED: f64 = 1.10;
const CALM: f64 = 1.05;
const FIRST_PATIENCE: u32 = 4;
const MAX_PATIENCE: u32 = 64;
const TRIAL_WINDOWS: u32 = 4;

#[derive(Resource, Debug, Clone, PartialEq)]
pub struct ScaleController {
    min: f32,
    max: f32,
    level: u8,
    frames: u16,
    total: f64,
    calm_windows: u32,
    patience: u32,
    trial_windows: u32,
}

impl ScaleController {
    #[must_use]
    pub fn new(min: f32, max: f32) -> Self {
        Self {
            min,
            max,
            level: 0,
            frames: 0,
            total: 0.0,
            calm_windows: 0,
            patience: FIRST_PATIENCE,
            trial_windows: 0,
        }
    }

    #[must_use]
    pub fn scale(&self) -> f32 {
        self.min + (self.max - self.min) * f32::from(self.level) / f32::from(LEVELS - 1)
    }

    pub fn record(&mut self, frame_time: f64, budget: f64) -> Option<f32> {
        self.total += frame_time;
        self.frames += 1;
        if self.frames < WINDOW_FRAMES {
            return None;
        }
        let mean = self.total / f64::from(WINDOW_FRAMES);
        self.total = 0.0;
        self.frames = 0;
        let on_trial = self.trial_windows > 0;
        self.trial_windows = self.trial_windows.saturating_sub(1);
        if mean > OVERLOADED * budget {
            self.calm_windows = 0;
            if on_trial {
                self.patience = (self.patience * 2).min(MAX_PATIENCE);
                self.trial_windows = 0;
            }
            return self.lower();
        }
        if on_trial && self.trial_windows == 0 {
            self.patience = FIRST_PATIENCE;
        }
        if mean >= CALM * budget {
            self.calm_windows = 0;
            return None;
        }
        self.calm_windows += 1;
        if self.calm_windows < self.patience {
            return None;
        }
        self.calm_windows = 0;
        self.raise()
    }

    fn lower(&mut self) -> Option<f32> {
        self.level = self.level.checked_sub(1)?;
        Some(self.scale())
    }

    fn raise(&mut self) -> Option<f32> {
        if self.level + 1 >= LEVELS {
            return None;
        }
        self.level += 1;
        self.trial_windows = TRIAL_WINDOWS;
        Some(self.scale())
    }
}

#[must_use]
pub fn frame_budget(present_mode: PresentMode, refresh_millihertz: Option<u32>) -> f64 {
    let vsync = matches!(
        present_mode,
        PresentMode::AutoVsync | PresentMode::Fifo | PresentMode::FifoRelaxed
    );
    refresh_millihertz
        .filter(|refresh| vsync && *refresh > 0)
        .map_or(TARGET_FRAME_TIME, |refresh| {
            TARGET_FRAME_TIME.max(1000.0 / f64::from(refresh))
        })
}

pub fn adjust_render_scale(
    time: Res<Time<Real>>,
    windows: Query<(&Window, Option<&OnMonitor>), With<PrimaryWindow>>,
    monitors: Query<(&Monitor, Has<PrimaryMonitor>)>,
    mut controller: ResMut<ScaleController>,
    mut scale: ResMut<RenderScale>,
    mut last_budget: Local<Option<f64>>,
) {
    let Ok((window, on_monitor)) = windows.single() else {
        return;
    };
    let refresh = on_monitor
        .and_then(|link| monitors.get(link.0).ok())
        .or_else(|| monitors.iter().find(|(_, primary)| *primary))
        .and_then(|(monitor, _)| monitor.refresh_rate_millihertz);
    let budget = frame_budget(window.present_mode, refresh);
    if last_budget.is_none_or(|last| (last - budget).abs() > 1e-6) {
        info!(
            "frame budget: {:.1} ms for {:?} at {:?} mHz",
            budget * 1000.0,
            window.present_mode,
            refresh
        );
        *last_budget = Some(budget);
    }
    if let Some(next) = controller.record(time.delta_secs_f64(), budget) {
        info!(
            "render scale: {:.0}% for a frame budget of {:.1} ms",
            next * 100.0,
            budget * 1000.0
        );
        scale.0 = next;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use bevy::window::{Monitor, OnMonitor, PresentMode, PrimaryWindow};

    use super::{ScaleController, frame_budget};
    use crate::{RenderScale, RenderScalePlugin};

    const BUDGET: f64 = 1.0 / 60.0;
    const SMOOTH: f64 = 1.0 / 60.0;
    const SLOW: f64 = 1.0 / 40.0;
    const WINDOW: usize = 30;
    const NO_CHANGE: [f32; 0] = [];

    fn feed(controller: &mut ScaleController, frame_time: f64, frames: usize) -> Vec<f32> {
        (0..frames)
            .filter_map(|_| controller.record(frame_time, BUDGET))
            .collect()
    }

    fn near(actual: &[f32], expected: &[f32]) -> bool {
        actual.len() == expected.len()
            && actual
                .iter()
                .zip(expected)
                .all(|(a, b)| (a - b).abs() < 1e-5)
    }

    #[test]
    fn the_controller_starts_at_the_lowest_scale() {
        let controller = ScaleController::new(0.6, 1.0);

        assert!((controller.scale() - 0.6).abs() < 1e-6);
    }

    #[test]
    fn four_calm_windows_raise_the_scale_by_one_step() {
        let mut controller = ScaleController::new(0.6, 1.0);

        assert_eq!(feed(&mut controller, SMOOTH, 4 * WINDOW - 1), NO_CHANGE);
        assert!(near(&feed(&mut controller, SMOOTH, 1), &[0.7]));
    }

    #[test]
    fn a_slow_window_lowers_the_scale_by_one_step() {
        let mut controller = ScaleController::new(0.6, 1.0);
        feed(&mut controller, SMOOTH, 4 * WINDOW);

        assert!(near(&feed(&mut controller, SLOW, WINDOW), &[0.6]));
    }

    #[test]
    fn the_scale_does_not_go_below_the_minimum() {
        let mut controller = ScaleController::new(0.6, 1.0);

        assert_eq!(feed(&mut controller, SLOW, 10 * WINDOW), NO_CHANGE);
    }

    #[test]
    fn calm_frames_raise_the_scale_to_the_maximum_and_not_above() {
        let mut controller = ScaleController::new(0.6, 1.0);

        let changes = feed(&mut controller, SMOOTH, 100 * WINDOW);

        assert!(near(&changes, &[0.7, 0.8, 0.9, 1.0]));
    }

    #[test]
    fn one_long_frame_does_not_lower_the_scale() {
        let mut controller = ScaleController::new(0.6, 1.0);
        feed(&mut controller, SMOOTH, 4 * WINDOW + WINDOW - 1);

        assert_eq!(feed(&mut controller, 0.05, 1), NO_CHANGE);
    }

    #[test]
    fn after_a_failed_raise_the_controller_waits_two_times_longer() {
        let mut controller = ScaleController::new(0.6, 1.0);
        feed(&mut controller, SMOOTH, 4 * WINDOW);
        feed(&mut controller, SLOW, WINDOW);

        assert_eq!(feed(&mut controller, SMOOTH, 8 * WINDOW - 1), NO_CHANGE);
        assert!(near(&feed(&mut controller, SMOOTH, 1), &[0.7]));
    }

    #[test]
    fn a_slow_window_soon_after_a_raise_also_counts_as_a_failed_raise() {
        let mut controller = ScaleController::new(0.6, 1.0);
        feed(&mut controller, SMOOTH, 4 * WINDOW);
        feed(&mut controller, SMOOTH, WINDOW);
        feed(&mut controller, SLOW, WINDOW);

        assert_eq!(feed(&mut controller, SMOOTH, 8 * WINDOW - 1), NO_CHANGE);
        assert!(near(&feed(&mut controller, SMOOTH, 1), &[0.7]));
    }

    #[test]
    fn without_vsync_the_budget_is_60_fps() {
        let budget = frame_budget(PresentMode::AutoNoVsync, Some(48_000));

        assert!((budget - 1.0 / 60.0).abs() < 1e-9);
    }

    #[test]
    fn with_vsync_on_a_48_hz_display_the_budget_is_one_refresh() {
        let budget = frame_budget(PresentMode::Fifo, Some(48_000));

        assert!((budget - 1.0 / 48.0).abs() < 1e-9);
    }

    #[test]
    fn with_vsync_on_a_fast_display_the_budget_stays_at_60_fps() {
        let budget = frame_budget(PresentMode::AutoVsync, Some(144_000));

        assert!((budget - 1.0 / 60.0).abs() < 1e-9);
    }

    #[test]
    fn with_vsync_and_no_known_refresh_rate_the_budget_is_60_fps() {
        let budget = frame_budget(PresentMode::Fifo, None);

        assert!((budget - 1.0 / 60.0).abs() < 1e-9);
    }

    fn render_scale_after(frame: Duration, refresh_millihertz: u32) -> (f32, f32) {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, RenderScalePlugin))
            .init_resource::<Assets<Image>>()
            .insert_resource(TimeUpdateStrategy::ManualDuration(frame));
        let monitor = app
            .world_mut()
            .spawn(Monitor {
                name: None,
                physical_height: 1080,
                physical_width: 1920,
                physical_position: IVec2::ZERO,
                refresh_rate_millihertz: Some(refresh_millihertz),
                scale_factor: 1.0,
                video_modes: Vec::new(),
            })
            .id();
        app.world_mut().spawn((
            Window {
                present_mode: PresentMode::Fifo,
                ..default()
            },
            PrimaryWindow,
            OnMonitor(monitor),
        ));
        let start = app.world().resource::<RenderScale>().0;

        for _ in 0..5 * WINDOW {
            app.update();
        }

        (start, app.world().resource::<RenderScale>().0)
    }

    #[test]
    fn on_a_48_hz_display_a_steady_48_fps_raises_the_render_scale() {
        let (start, end) = render_scale_after(Duration::from_millis(20), 48_000);

        assert!(end > start);
    }

    #[test]
    fn on_a_60_hz_display_a_steady_50_fps_does_not_raise_the_render_scale() {
        let (start, end) = render_scale_after(Duration::from_millis(20), 60_000);

        assert!((end - start).abs() < 1e-6);
    }
}
