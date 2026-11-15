use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::{FrameSummary, GpuPassSummary};

#[derive(Serialize)]
struct Report<'a> {
    frames: usize,
    mean_fps: f64,
    p50_ms: f64,
    p95_ms: f64,
    p99_ms: f64,
    one_percent_low_fps: f64,
    gpu_ms_per_frame: Option<f64>,
    gpu_passes: Vec<PassTime<'a>>,
}

#[derive(Serialize)]
struct PassTime<'a> {
    pass: &'a str,
    ms: f64,
}

fn hundredths(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

pub(crate) fn report_json(
    frames: &FrameSummary,
    gpu: Option<&GpuPassSummary>,
) -> serde_json::Result<String> {
    let report = Report {
        frames: frames.frames,
        mean_fps: hundredths(frames.mean_fps),
        p50_ms: hundredths(frames.p50_ms),
        p95_ms: hundredths(frames.p95_ms),
        p99_ms: hundredths(frames.p99_ms),
        one_percent_low_fps: hundredths(frames.one_percent_low_fps),
        gpu_ms_per_frame: gpu.map(|gpu| hundredths(gpu.total)),
        gpu_passes: gpu.map_or_else(Vec::new, |gpu| {
            gpu.passes
                .iter()
                .map(|(pass, ms)| PassTime {
                    pass,
                    ms: hundredths(*ms),
                })
                .collect()
        }),
    };
    serde_json::to_string_pretty(&report)
}

pub(crate) fn write_report(folder: &Path, json: &str, time: SystemTime) -> io::Result<PathBuf> {
    fs::create_dir_all(folder)?;
    let seconds = time
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    let path = folder.join(format!("benchmark-{seconds}.json"));
    fs::write(&path, json)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    use serde_json::Value;

    use super::{report_json, write_report};
    use crate::{FrameSummary, GpuPassSummary};

    fn frames() -> FrameSummary {
        FrameSummary {
            frames: 1200,
            mean_fps: 58.31,
            p50_ms: 16.66,
            p95_ms: 17.94,
            p99_ms: 25.12,
            one_percent_low_fps: 38.24,
        }
    }

    fn parse(json: &str) -> Value {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn a_report_has_the_frame_times_and_the_gpu_passes() {
        let gpu = GpuPassSummary {
            total: 14.5,
            passes: vec![
                ("main_opaque_pass_3d".to_owned(), 7.25),
                ("fxaa".to_owned(), 1.3),
            ],
        };

        let json = parse(&report_json(&frames(), Some(&gpu)).unwrap());

        assert_eq!(json["frames"], 1200);
        assert_eq!(json["mean_fps"], 58.31);
        assert_eq!(json["p50_ms"], 16.66);
        assert_eq!(json["p95_ms"], 17.94);
        assert_eq!(json["p99_ms"], 25.12);
        assert_eq!(json["one_percent_low_fps"], 38.24);
        assert_eq!(json["gpu_ms_per_frame"], 14.5);
        assert_eq!(json["gpu_passes"][0]["pass"], "main_opaque_pass_3d");
        assert_eq!(json["gpu_passes"][0]["ms"], 7.25);
        assert_eq!(json["gpu_passes"][1]["pass"], "fxaa");
    }

    #[test]
    fn a_report_rounds_to_hundredths() {
        let mut summary = frames();
        summary.p50_ms = f64::from(16.4_f32);

        let json = parse(&report_json(&summary, None).unwrap());

        assert_eq!(json["p50_ms"], 16.4);
    }

    #[test]
    fn a_report_without_gpu_timers_has_no_gpu_values() {
        let json = parse(&report_json(&frames(), None).unwrap());

        assert!(json["gpu_ms_per_frame"].is_null());
        assert_eq!(json["gpu_passes"], serde_json::json!([]));
    }

    #[test]
    fn a_report_file_has_the_time_of_the_run_in_its_name() {
        let directory = tempfile::tempdir().unwrap();
        let folder = directory.path().join("benchmarks");
        let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1_763_200_000);

        let path = write_report(&folder, "{}", time).unwrap();

        assert_eq!(path, folder.join("benchmark-1763200000.json"));
        assert_eq!(std::fs::read_to_string(path).unwrap(), "{}");
    }
}
