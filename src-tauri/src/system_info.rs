use serde::Serialize;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use sysinfo::{DiskRefreshKind, Disks, System};
use tauri::State;

const DISK_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Serialize)]
pub struct DiskSnapshot {
    pub name: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct SystemSnapshot {
    pub computer_name: Option<String>,
    pub os_version: Option<String>,
    pub cpu_arch: Option<String>,
    pub logical_threads: usize,
    pub total_memory: u64,
    pub used_memory: u64,
    pub disks: Vec<DiskSnapshot>,
}

pub struct SystemInfoState {
    collector: Mutex<SystemInfoCollector>,
}

struct SystemInfoCollector {
    system: System,
    disks: Disks,
    last_disk_refresh: Option<Instant>,
}

impl SystemInfoState {
    pub fn new() -> Self {
        Self {
            collector: Mutex::new(SystemInfoCollector {
                // Keep a single lightweight System instance. It is never asked
                // to enumerate processes or CPU usage.
                system: System::new(),
                disks: Disks::new(),
                last_disk_refresh: None,
            }),
        }
    }
}

fn disk_refresh_due(last_refresh: Instant, now: Instant) -> bool {
    now.saturating_duration_since(last_refresh) >= DISK_REFRESH_INTERVAL
}

#[tauri::command]
pub fn get_system_snapshot(state: State<'_, SystemInfoState>) -> Result<SystemSnapshot, String> {
    let mut collector = state
        .collector
        .lock()
        .map_err(|_| "System information is temporarily unavailable.".to_string())?;

    collector.system.refresh_memory();

    let now = Instant::now();
    let should_refresh_disks = collector
        .last_disk_refresh
        .map(|last_refresh| disk_refresh_due(last_refresh, now))
        .unwrap_or(true);

    if should_refresh_disks {
        // Storage figures change slowly. Refresh only capacity fields, and
        // avoid rebuilding the disk collector on every dashboard update.
        collector.disks.refresh_specifics(
            true,
            DiskRefreshKind::nothing().with_storage(),
        );
        collector.last_disk_refresh = Some(now);
    }

    let disks = collector
        .disks
        .list()
        .iter()
        .map(|disk| DiskSnapshot {
            name: disk.mount_point().display().to_string(),
            total_bytes: disk.total_space(),
            available_bytes: disk.available_space(),
        })
        .collect();

    Ok(SystemSnapshot {
        computer_name: System::host_name(),
        os_version: System::long_os_version(),
        cpu_arch: Some(System::cpu_arch()),
        logical_threads: std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(0),
        total_memory: collector.system.total_memory(),
        used_memory: collector.system.used_memory(),
        disks,
    })
}

#[cfg(test)]
mod tests {
    use super::SystemSnapshot;

    #[test]
    fn system_snapshot_serializes_a_present_cpu_architecture() {
        let snapshot = SystemSnapshot {
            computer_name: Some("HADOC-PC".into()),
            os_version: Some("Windows".into()),
            cpu_arch: Some("x86_64".into()),
            logical_threads: 4,
            total_memory: 1024,
            used_memory: 512,
            disks: Vec::new(),
        };

        let value = serde_json::to_value(snapshot).unwrap();

        assert_eq!(value["cpu_arch"], "x86_64");
        assert_eq!(value["computer_name"], "HADOC-PC");
    }

    #[test]
    fn system_snapshot_serializes_optional_values_and_disk_list() {
        let snapshot = SystemSnapshot {
            computer_name: None,
            os_version: None,
            cpu_arch: None,
            logical_threads: 0,
            total_memory: 0,
            used_memory: 0,
            disks: Vec::new(),
        };

        let value = serde_json::to_value(snapshot).unwrap();

        assert!(value["computer_name"].is_null());
        assert!(value["os_version"].is_null());
        assert!(value["cpu_arch"].is_null());
        assert_eq!(value["logical_threads"], 0);
        assert_eq!(value["disks"], serde_json::json!([]));
    }

    #[test]
    fn disk_refresh_is_throttled_until_the_interval_has_elapsed() {
        use std::time::{Duration, Instant};

        let last_refresh = Instant::now();

        assert!(!super::disk_refresh_due(
            last_refresh,
            last_refresh + Duration::from_secs(29)
        ));
        assert!(super::disk_refresh_due(
            last_refresh,
            last_refresh + Duration::from_secs(30)
        ));
    }
}
