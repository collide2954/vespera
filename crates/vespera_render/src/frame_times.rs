use std::fmt;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FrameTimes {
    seconds: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrameSummary {
    pub frames: usize,
    pub mean_fps: f64,
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub one_percent_low_fps: f64,
}

impl FrameTimes {
    pub fn record(&mut self, seconds: f32) {
        if seconds.is_finite() && seconds > 0.0 {
            self.seconds.push(seconds);
        }
    }

    #[must_use]
    pub fn summary(&self) -> Option<FrameSummary> {
        if self.seconds.is_empty() {
            return None;
        }
        let mut sorted = self.seconds.clone();
        sorted.sort_by(f32::total_cmp);
        let frames = sorted.len();
        let slowest = &sorted[frames - frames.div_ceil(100)..];
        Some(FrameSummary {
            frames,
            mean_fps: count(&sorted) / total(&sorted),
            p50_ms: percentile_ms(&sorted, 50),
            p95_ms: percentile_ms(&sorted, 95),
            p99_ms: percentile_ms(&sorted, 99),
            one_percent_low_fps: count(slowest) / total(slowest),
        })
    }
}

impl fmt::Display for FrameSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} frames, mean {:.1} FPS, p50 {:.1} ms, p95 {:.1} ms, p99 {:.1} ms, 1% low {:.1} FPS",
            self.frames,
            self.mean_fps,
            self.p50_ms,
            self.p95_ms,
            self.p99_ms,
            self.one_percent_low_fps
        )
    }
}

fn count(seconds: &[f32]) -> f64 {
    f64::from(u32::try_from(seconds.len()).unwrap_or(u32::MAX))
}

fn total(seconds: &[f32]) -> f64 {
    seconds.iter().map(|value| f64::from(*value)).sum()
}

fn percentile_ms(sorted: &[f32], percent: usize) -> f64 {
    let rank = (sorted.len() * percent).div_ceil(100).max(1);
    f64::from(sorted[rank - 1]) * 1000.0
}

#[cfg(test)]
mod tests {
    use super::{FrameSummary, FrameTimes};

    #[test]
    fn a_summary_reads_as_one_line() {
        let summary = FrameSummary {
            frames: 1200,
            mean_fps: 58.31,
            p50_ms: 16.66,
            p95_ms: 17.94,
            p99_ms: 25.12,
            one_percent_low_fps: 38.24,
        };

        assert_eq!(
            summary.to_string(),
            "1200 frames, mean 58.3 FPS, p50 16.7 ms, p95 17.9 ms, p99 25.1 ms, 1% low 38.2 FPS"
        );
    }

    fn frames(seconds: &[f32]) -> FrameTimes {
        let mut times = FrameTimes::default();
        for value in seconds {
            times.record(*value);
        }
        times
    }

    fn close(actual: f64, expected: f64) -> bool {
        (actual - expected).abs() < 1e-3
    }

    #[test]
    fn no_frames_give_no_summary() {
        assert_eq!(FrameTimes::default().summary(), None);
    }

    #[test]
    fn steady_frames_give_the_same_value_for_each_percentile() {
        let summary = frames(&[0.02; 50]).summary().unwrap();

        assert_eq!(summary.frames, 50);
        assert!(close(summary.mean_fps, 50.0));
        assert!(close(summary.p50_ms, 20.0));
        assert!(close(summary.p95_ms, 20.0));
        assert!(close(summary.p99_ms, 20.0));
        assert!(close(summary.one_percent_low_fps, 50.0));
    }

    #[test]
    fn percentiles_use_the_nearest_rank() {
        let seconds: Vec<f32> = (1_u16..=100)
            .map(|millis| f32::from(millis) / 1000.0)
            .collect();

        let summary = frames(&seconds).summary().unwrap();

        assert!(close(summary.p50_ms, 50.0));
        assert!(close(summary.p95_ms, 95.0));
        assert!(close(summary.p99_ms, 99.0));
    }

    #[test]
    fn the_mean_frame_rate_uses_the_total_time() {
        let summary = frames(&[0.01, 0.03]).summary().unwrap();

        assert!(close(summary.mean_fps, 50.0));
    }

    #[test]
    fn the_one_percent_low_is_the_frame_rate_of_the_slowest_frames() {
        let mut seconds = vec![0.01; 198];
        seconds.extend([0.1, 0.1]);

        let summary = frames(&seconds).summary().unwrap();

        assert!(close(summary.one_percent_low_fps, 10.0));
    }

    #[test]
    fn frames_without_time_are_ignored() {
        let summary = frames(&[0.0, 0.02, f32::NAN, -0.01]).summary().unwrap();

        assert_eq!(summary.frames, 1);
    }
}
