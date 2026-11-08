use serde::Serialize;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const CPU_SAMPLE_DURATION: Duration = Duration::from_millis(180);
const MEMORY_BUFFER_BYTES: usize = 8 * 1024 * 1024;
const MEMORY_PASSES: usize = 8;
const DISK_SAMPLE_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Serialize)]
pub struct BenchmarkResult {
    pub logical_threads: usize,
    pub cpu_single_m_iterations_s: f64,
    pub cpu_multi_m_iterations_s: f64,
    pub memory_copy_mb_s: f64,
    pub disk_write_mb_s: f64,
    pub disk_read_mb_s: f64,
    pub duration_ms: u64,
    pub disk_read_note: String,
}

#[derive(Debug)]
struct DiskSample {
    write_mb_s: f64,
    read_mb_s: f64,
}

fn cpu_sample(duration: Duration) -> u64 {
    let start = Instant::now();
    let mut value = 0x9e37_79b9_7f4a_7c15u64;
    let mut iterations = 0u64;

    while start.elapsed() < duration {
        for _ in 0..1024 {
            value = value
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            value ^= value.rotate_left(17);
            iterations += 1;
        }
    }

    std::hint::black_box(value);
    iterations
}

fn memory_copy_sample(buffer_bytes: usize, passes: usize) -> f64 {
    if buffer_bytes == 0 || passes == 0 {
        return 0.0;
    }

    let source = vec![0x5au8; buffer_bytes];
    let mut destination = vec![0u8; buffer_bytes];
    let start = Instant::now();
    for _ in 0..passes {
        destination.copy_from_slice(&source);
        std::hint::black_box(destination.as_slice());
    }
    let elapsed = start.elapsed().as_secs_f64().max(0.000_001);
    (buffer_bytes as f64 * passes as f64 * 2.0 / (1024.0 * 1024.0)) / elapsed
}

fn disk_sample(directory: &Path, bytes: usize) -> Result<DiskSample, String> {
    fs::create_dir_all(directory)
        .map_err(|error| format!("Cannot access temporary benchmark folder: {error}"))?;
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_path: PathBuf = directory.join(format!(
        "hadoc-benchmark-{}-{unique}.tmp",
        std::process::id()
    ));

    let result = (|| {
        let block = vec![0xA5u8; 1024 * 1024];
        let write_start = Instant::now();
        let mut file = File::create(&file_path)
            .map_err(|error| format!("Cannot create benchmark file: {error}"))?;
        let mut written = 0usize;
        while written < bytes {
            let chunk = (bytes - written).min(block.len());
            file.write_all(&block[..chunk])
                .map_err(|error| format!("Disk write benchmark failed: {error}"))?;
            written += chunk;
        }
        file.sync_all()
            .map_err(|error| format!("Cannot flush benchmark file: {error}"))?;
        let write_seconds = write_start.elapsed().as_secs_f64().max(0.000_001);

        drop(file);
        let read_start = Instant::now();
        let mut file = File::open(&file_path)
            .map_err(|error| format!("Cannot read benchmark file: {error}"))?;
        let mut buffer = vec![0u8; 256 * 1024];
        let mut read_bytes = 0usize;
        loop {
            let count = file
                .read(&mut buffer)
                .map_err(|error| format!("Disk read benchmark failed: {error}"))?;
            if count == 0 {
                break;
            }
            read_bytes += count;
            std::hint::black_box(&buffer[..count]);
        }
        let read_seconds = read_start.elapsed().as_secs_f64().max(0.000_001);
        if read_bytes == 0 {
            return Err("Disk benchmark returned no data.".into());
        }

        let megabytes = bytes as f64 / (1024.0 * 1024.0);
        Ok(DiskSample {
            write_mb_s: megabytes / write_seconds,
            read_mb_s: (read_bytes as f64 / (1024.0 * 1024.0)) / read_seconds,
        })
    })();

    let _ = fs::remove_file(&file_path);
    result
}

fn run_benchmark() -> Result<BenchmarkResult, String> {
    let total_start = Instant::now();
    let logical_threads = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .max(1);
    let workers = logical_threads.min(8);

    let cpu_single_start = Instant::now();
    let single_iterations = cpu_sample(CPU_SAMPLE_DURATION);
    let single_seconds = cpu_single_start.elapsed().as_secs_f64().max(0.000_001);

    let multi_start = Instant::now();
    let handles = (0..workers)
        .map(|_| thread::spawn(|| cpu_sample(CPU_SAMPLE_DURATION)))
        .collect::<Vec<_>>();
    let multi_iterations = handles
        .into_iter()
        .map(|handle| handle.join().unwrap_or(0))
        .sum::<u64>();
    let multi_seconds = multi_start.elapsed().as_secs_f64().max(0.000_001);

    let memory_copy_mb_s = memory_copy_sample(MEMORY_BUFFER_BYTES, MEMORY_PASSES);
    let disk = disk_sample(&std::env::temp_dir(), DISK_SAMPLE_BYTES)?;

    Ok(BenchmarkResult {
        logical_threads,
        cpu_single_m_iterations_s: single_iterations as f64 / single_seconds / 1_000_000.0,
        cpu_multi_m_iterations_s: multi_iterations as f64 / multi_seconds / 1_000_000.0,
        memory_copy_mb_s,
        disk_write_mb_s: disk.write_mb_s,
        disk_read_mb_s: disk.read_mb_s,
        duration_ms: total_start.elapsed().as_millis().min(u64::MAX as u128) as u64,
        disk_read_note: "Sequential read; may include Windows file cache when available".into(),
    })
}

#[tauri::command]
pub async fn run_quick_benchmark() -> Result<BenchmarkResult, String> {
    tauri::async_runtime::spawn_blocking(run_benchmark)
        .await
        .map_err(|error| format!("Benchmark worker failed: {error}"))?
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn cpu_sample_performs_measurable_work() {
        assert!(super::cpu_sample(Duration::from_millis(5)) > 0);
    }

    #[test]
    fn memory_copy_sample_returns_positive_bandwidth() {
        let bandwidth = super::memory_copy_sample(256 * 1024, 2);
        assert!(bandwidth.is_finite() && bandwidth > 0.0);
    }

    #[test]
    fn disk_sample_measures_io_and_removes_its_temporary_file() {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let folder = std::env::temp_dir().join(format!("hadoc-benchmark-{unique}"));
        fs::create_dir_all(&folder).unwrap();

        let sample = super::disk_sample(&folder, 128 * 1024).unwrap();

        assert!(sample.write_mb_s.is_finite() && sample.write_mb_s > 0.0);
        assert!(sample.read_mb_s.is_finite() && sample.read_mb_s > 0.0);
        assert_eq!(fs::read_dir(&folder).unwrap().count(), 0);
        fs::remove_dir(&folder).unwrap();
    }
}
