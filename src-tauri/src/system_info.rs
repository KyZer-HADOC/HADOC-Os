use serde::Serialize;
use sysinfo::{Disks, System};

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

#[tauri::command]
pub fn get_system_snapshot() -> SystemSnapshot {
    let mut system = System::new();
    system.refresh_memory();

    let disks = Disks::new_with_refreshed_list()
        .iter()
        .map(|disk| DiskSnapshot {
            name: disk.mount_point().display().to_string(),
            total_bytes: disk.total_space(),
            available_bytes: disk.available_space(),
        })
        .collect();

    SystemSnapshot {
        computer_name: System::host_name(),
        os_version: System::long_os_version(),
        cpu_arch: System::cpu_arch(),
        logical_threads: std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(0),
        total_memory: system.total_memory(),
        used_memory: system.used_memory(),
        disks,
    }
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
}
