use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use bevy::diagnostic::DiagnosticsStore;
use bevy::platform::time::Instant;

#[derive(Debug, Default, Clone)]
pub struct GpuPassTimes {
    totals: BTreeMap<String, f64>,
    frames: u32,
    last: Option<Instant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GpuPassSummary {
    pub total: f64,
    pub passes: Vec<(String, f64)>,
}

impl GpuPassTimes {
    pub fn record(&mut self, store: &DiagnosticsStore) {
        let mut new_frames = BTreeSet::new();
        for diagnostic in store.iter() {
            let Some(pass) = pass_name(diagnostic.path().as_str()) else {
                continue;
            };
            for measurement in diagnostic
                .measurements()
                .filter(|measurement| self.last.is_none_or(|last| measurement.time > last))
            {
                *self.totals.entry(pass.to_owned()).or_default() += measurement.value;
                new_frames.insert(measurement.time);
            }
        }
        if let Some(&newest) = new_frames.last() {
            self.last = Some(newest);
        }
        self.frames = self
            .frames
            .saturating_add(u32::try_from(new_frames.len()).unwrap_or(u32::MAX));
    }

    #[must_use]
    pub fn summary(&self) -> Option<GpuPassSummary> {
        if self.frames == 0 {
            return None;
        }
        let frames = f64::from(self.frames);
        let mut passes: Vec<(String, f64)> = self
            .totals
            .iter()
            .map(|(name, total)| (name.clone(), total / frames))
            .collect();
        passes.sort_by(|a, b| b.1.total_cmp(&a.1));
        let total = passes.iter().map(|(_, time)| time).sum();
        Some(GpuPassSummary { total, passes })
    }
}

fn pass_name(path: &str) -> Option<&str> {
    path.strip_prefix("render/")?
        .strip_suffix("/elapsed_gpu")
        .filter(|name| !name.contains('/'))
}

impl fmt::Display for GpuPassSummary {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "gpu {:.1} ms per frame:", self.total)?;
        for (index, (name, time)) in self.passes.iter().enumerate() {
            let separator = if index == 0 { " " } else { ", " };
            write!(formatter, "{separator}{name} {time:.1}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::diagnostic::{Diagnostic, DiagnosticMeasurement, DiagnosticPath, DiagnosticsStore};
    use bevy::platform::time::Instant;

    use super::GpuPassTimes;

    fn store_with(measurements: &[(&'static str, u64, f64)]) -> DiagnosticsStore {
        let start = Instant::now();
        let mut store = DiagnosticsStore::default();
        for &(path, frame, value) in measurements {
            let path = DiagnosticPath::new(path);
            if store.get(&path).is_none() {
                store.add(Diagnostic::new(path.clone()));
            }
            store
                .get_mut(&path)
                .expect("the diagnostic exists")
                .add_measurement(DiagnosticMeasurement {
                    time: start + Duration::from_millis(frame),
                    value,
                });
        }
        store
    }

    fn summary_of(store: &DiagnosticsStore) -> Option<String> {
        let mut times = GpuPassTimes::default();
        times.record(store);
        times.summary().map(|summary| summary.to_string())
    }

    #[test]
    fn the_time_of_a_pass_is_its_mean_over_the_frames() {
        let store = store_with(&[
            ("render/fxaa/elapsed_gpu", 1, 1.0),
            ("render/fxaa/elapsed_gpu", 2, 3.0),
        ]);

        assert_eq!(
            summary_of(&store).as_deref(),
            Some("gpu 2.0 ms per frame: fxaa 2.0")
        );
    }

    #[test]
    fn two_spans_with_the_same_name_in_one_frame_add_up() {
        let store = store_with(&[
            ("render/upscaling/elapsed_gpu", 1, 1.0),
            ("render/upscaling/elapsed_gpu", 1, 2.0),
        ]);

        assert_eq!(
            summary_of(&store).as_deref(),
            Some("gpu 3.0 ms per frame: upscaling 3.0")
        );
    }

    #[test]
    fn nested_spans_and_cpu_times_do_not_count() {
        let store = store_with(&[
            ("render/cas/elapsed_gpu", 1, 2.0),
            ("render/cas/cas/elapsed_gpu", 1, 1.5),
            ("render/cas/elapsed_cpu", 1, 0.1),
        ]);

        assert_eq!(
            summary_of(&store).as_deref(),
            Some("gpu 2.0 ms per frame: cas 2.0")
        );
    }

    #[test]
    fn the_slowest_pass_comes_first() {
        let store = store_with(&[
            ("render/fxaa/elapsed_gpu", 1, 1.0),
            ("render/main_opaque_pass_3d/elapsed_gpu", 1, 7.0),
        ]);

        assert_eq!(
            summary_of(&store).as_deref(),
            Some("gpu 8.0 ms per frame: main_opaque_pass_3d 7.0, fxaa 1.0")
        );
    }

    #[test]
    fn a_second_record_counts_only_the_new_frames() {
        let store = store_with(&[("render/fxaa/elapsed_gpu", 1, 2.0)]);
        let mut times = GpuPassTimes::default();

        times.record(&store);
        times.record(&store);

        assert_eq!(
            times
                .summary()
                .map(|summary| summary.to_string())
                .as_deref(),
            Some("gpu 2.0 ms per frame: fxaa 2.0")
        );
    }

    #[test]
    fn without_gpu_times_there_is_no_summary() {
        let store = store_with(&[("render/fxaa/elapsed_cpu", 1, 0.1)]);

        assert_eq!(summary_of(&store), None);
    }
}
