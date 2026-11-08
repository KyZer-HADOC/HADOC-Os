#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
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
        let _: Option<PathBuf> = None;
    }
}
