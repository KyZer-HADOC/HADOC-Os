#[cfg(test)]
mod tests {
    use super::SystemSnapshot;

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
