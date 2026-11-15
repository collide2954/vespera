use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use bevy::render::renderer::RenderDevice;
use serde::Serialize;
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};

use crate::{FrameSummary, GpuPassSummary};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct Memory {
    pub(crate) process_mb: Option<f64>,
    pub(crate) gpu_mb: Option<f64>,
}

impl Memory {
    pub(crate) fn measure(device: Option<&RenderDevice>) -> Self {
        Self {
            process_mb: process_memory_mb(),
            gpu_mb: device.and_then(gpu_memory_mb),
        }
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "memory: process {}, GPU {}",
            amount(self.process_mb),
            amount(self.gpu_mb)
        )
    }
}

fn amount(megabytes: Option<f64>) -> String {
    megabytes.map_or_else(|| "unknown".to_owned(), |value| format!("{value:.0} MB"))
}

pub(crate) fn process_memory_mb() -> Option<f64> {
    let pid = sysinfo::get_current_pid().ok()?;
    let mut system = System::new();
    system.refresh_processes_specifics(
        ProcessesToUpdate::Some(&[pid]),
        true,
        ProcessRefreshKind::nothing().with_memory(),
    );
    system
        .process(pid)
        .map(|process| megabytes(process.memory()))
}

fn gpu_memory_mb(device: &RenderDevice) -> Option<f64> {
    device
        .wgpu_device()
        .generate_allocator_report()
        .map(|report| megabytes(report.total_reserved_bytes))
}

fn megabytes(bytes: u64) -> f64 {
    let kibibytes = u32::try_from(bytes / 1024).unwrap_or(u32::MAX);
    f64::from(kibibytes) / 1024.0
}

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
    process_memory_mb: Option<f64>,
    gpu_memory_mb: Option<f64>,
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
    memory: &Memory,
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
        process_memory_mb: memory.process_mb.map(hundredths),
        gpu_memory_mb: memory.gpu_mb.map(hundredths),
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

    use super::{Memory, process_memory_mb, report_json, write_report};
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

        let json = parse(&report_json(&frames(), Some(&gpu), &Memory::default()).unwrap());

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

        let json = parse(&report_json(&summary, None, &Memory::default()).unwrap());

        assert_eq!(json["p50_ms"], 16.4);
    }

    #[test]
    fn a_report_without_gpu_timers_has_no_gpu_values() {
        let json = parse(&report_json(&frames(), None, &Memory::default()).unwrap());

        assert!(json["gpu_ms_per_frame"].is_null());
        assert_eq!(json["gpu_passes"], serde_json::json!([]));
        assert!(json["gpu_memory_mb"].is_null());
    }

    #[test]
    fn a_report_has_the_memory_of_the_process_and_the_gpu() {
        let memory = Memory {
            process_mb: Some(812.345),
            gpu_mb: Some(230.0),
        };

        let json = parse(&report_json(&frames(), None, &memory).unwrap());

        assert_eq!(json["process_memory_mb"], 812.35);
        assert_eq!(json["gpu_memory_mb"], 230.0);
    }

    #[test]
    fn the_memory_line_names_an_unknown_value() {
        let memory = Memory {
            process_mb: Some(812.4),
            gpu_mb: None,
        };

        assert_eq!(memory.to_string(), "memory: process 812 MB, GPU unknown");
    }

    #[test]
    fn the_test_process_uses_some_memory() {
        assert!(process_memory_mb().is_some_and(|megabytes| megabytes > 1.0));
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
