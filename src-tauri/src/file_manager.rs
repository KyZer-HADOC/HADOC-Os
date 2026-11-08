use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct DirectoryListing {
    pub path: String,
    pub parent: Option<String>,
    pub entries: Vec<FileEntry>,
}

#[derive(Debug, Serialize)]
pub struct QuickLocation {
    pub name: String,
    pub path: String,
    pub icon: String,
    pub group: String,
}

fn read_directory(path: &Path) -> Result<DirectoryListing, String> {
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Cannot access {}: {error}", path.display()))?;

    if !canonical.is_dir() {
        return Err(format!("{} is not a folder.", canonical.display()));
    }

    let mut entries = Vec::new();
    let directory = fs::read_dir(&canonical)
        .map_err(|error| format!("Cannot read {}: {error}", canonical.display()))?;

    for entry in directory {
        let Ok(entry) = entry else { continue };
        let entry_path = entry.path();
        let Ok(metadata) = entry.metadata() else { continue };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());

        entries.push(FileEntry {
            name: entry.file_name().to_string_lossy().into_owned(),
            path: entry_path.to_string_lossy().into_owned(),
            is_dir: metadata.is_dir(),
            size: if metadata.is_file() { Some(metadata.len()) } else { None },
            modified,
        });
    }

    entries.sort_by(|left, right| {
        right
            .is_dir
            .cmp(&left.is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.name.cmp(&right.name))
    });

    Ok(DirectoryListing {
        path: canonical.to_string_lossy().into_owned(),
        parent: canonical.parent().map(|parent| parent.to_string_lossy().into_owned()),
        entries,
    })
}

#[tauri::command]
pub fn list_directory(path: Option<String>) -> Result<DirectoryListing, String> {
    let target = match path {
        Some(path) if !path.trim().is_empty() => PathBuf::from(path),
        Some(_) => return Err("Enter a folder path.".into()),
        None => default_home_directory()?,
    };

    read_directory(&target)
}

#[tauri::command]
pub fn get_quick_locations() -> Vec<QuickLocation> {
    let Some(home) = home_directory() else {
        return Vec::new();
    };

    let mut locations = Vec::new();
    if home.is_dir() {
        locations.push(QuickLocation {
            name: "Home".into(),
            path: home.to_string_lossy().into_owned(),
            icon: "⌂".into(),
            group: "Places".into(),
        });
    }

    let folders = [
        ("Desktop", "Desktop", "▣"),
        ("Documents", "Documents", "▤"),
        ("Downloads", "Downloads", "⇩"),
        ("Pictures", "Pictures", "▧"),
        ("Music", "Music", "♫"),
        ("Videos", "Videos", "▶"),
    ];

    for (name, folder, icon) in folders {
        let candidate = home.join(folder);
        if candidate.is_dir() {
            locations.push(QuickLocation {
                name: name.into(),
                path: candidate.to_string_lossy().into_owned(),
                icon: icon.into(),
                group: "Places".into(),
            });
        }
    }

    #[cfg(target_os = "windows")]
    for letter in b'A'..=b'Z' {
        let drive = PathBuf::from(format!("{}:\\", letter as char));
        if drive.is_dir() {
            locations.push(QuickLocation {
                name: format!("{}:", letter as char),
                path: drive.to_string_lossy().into_owned(),
                icon: "▰".into(),
                group: "Drives".into(),
            });
        }
    }

    locations
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    let target = PathBuf::from(path)
        .canonicalize()
        .map_err(|error| format!("Cannot open selected item: {error}"))?;

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;

        #[link(name = "shell32")]
        extern "system" {
            fn ShellExecuteW(
                hwnd: *mut std::ffi::c_void,
                operation: *const u16,
                file: *const u16,
                parameters: *const u16,
                directory: *const u16,
                show_command: i32,
            ) -> *mut std::ffi::c_void;
        }

        let operation: Vec<u16> = std::ffi::OsStr::new("open")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let target_wide: Vec<u16> = target.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let result = unsafe {
            ShellExecuteW(
                ptr::null_mut(),
                operation.as_ptr(),
                target_wide.as_ptr(),
                ptr::null(),
                ptr::null(),
                1,
            )
        };

        if result as isize <= 32 {
            return Err(format!("Windows could not open {}.", target.display()));
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = target;
        Err("Opening items from HADOC OS is currently supported on Windows only.".into())
    }
}

fn home_directory() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

fn default_home_directory() -> Result<PathBuf, String> {
    home_directory().ok_or_else(|| "Windows user folder could not be found.".into())
}

#[cfg(test)]
mod tests {
    use super::read_directory;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock must be after Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "hadoc-file-manager-{}-{unique}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("temporary directory should be created");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn read_directory_sorts_folders_before_files_and_reports_file_sizes() {
        let temp = TempDirectory::new();
        fs::create_dir(temp.path().join("z-folder")).unwrap();
        fs::write(temp.path().join("b.txt"), b"12345").unwrap();
        fs::write(temp.path().join("a.txt"), b"abc").unwrap();

        let listing = read_directory(temp.path()).unwrap();

        assert_eq!(listing.entries.len(), 3);
        assert_eq!(listing.entries[0].name, "z-folder");
        assert!(listing.entries[0].is_dir);
        assert_eq!(listing.entries[1].name, "a.txt");
        assert_eq!(listing.entries[1].size, Some(3));
        assert_eq!(listing.entries[2].name, "b.txt");
        assert_eq!(listing.entries[2].size, Some(5));
    }

    #[test]
    fn read_directory_rejects_a_regular_file() {
        let temp = TempDirectory::new();
        let file = temp.path().join("not-a-folder.txt");
        fs::write(&file, b"data").unwrap();

        assert!(read_directory(&file).is_err());
    }

    #[test]
    fn read_directory_returns_an_empty_list_for_an_empty_folder() {
        let temp = TempDirectory::new();

        let listing = read_directory(temp.path()).unwrap();

        assert!(listing.entries.is_empty());
    }

    #[test]
    fn logical_drive_mask_includes_only_assigned_drive_letters() {
        let roots = super::logical_drive_roots(0b1_0100);

        assert_eq!(roots, vec![PathBuf::from(r"C:\"), PathBuf::from(r"E:\")]);
    }

    #[test]
    fn this_pc_virtual_listing_contains_each_assigned_drive() {
        let roots = vec![PathBuf::from(r"C:\"), PathBuf::from(r"D:\")];

        let listing = super::this_pc_listing_from_roots(&roots);

        assert_eq!(listing.path, "__HADOC_THIS_PC__");
        assert_eq!(listing.parent, None);
        assert_eq!(listing.entries.len(), 2);
        assert_eq!(listing.entries[0].name, "C:");
        assert_eq!(listing.entries[1].name, "D:");
        assert!(listing.entries.iter().all(|entry| entry.is_dir));
    }

    #[test]
    fn quick_locations_skip_missing_paths_and_deduplicate_existing_paths() {
        let temp = TempDirectory::new();
        let mut locations = Vec::new();

        super::push_existing_location(
            &mut locations, "Home", temp.path(), "⌂", "User"
        );
        super::push_existing_location(
            &mut locations, "Home duplicate", temp.path(), "⌂", "User"
        );
        super::push_existing_location(
            &mut locations, "Missing", &temp.path().join("not-created"), "▤", "User"
        );

        assert_eq!(locations.len(), 1);
        assert_eq!(locations[0].name, "Home");
    }

    #[test]
    fn directory_pages_keep_global_folder_first_order() {
        let temp = TempDirectory::new();
        fs::create_dir(temp.path().join("Z-folder")).unwrap();
        fs::write(temp.path().join("c.txt"), b"c").unwrap();
        fs::write(temp.path().join("a.txt"), b"a").unwrap();
        fs::write(temp.path().join("b.txt"), b"b").unwrap();

        let first = super::read_directory_page(temp.path(), 0, 2, None).unwrap();
        let second = super::read_directory_page(temp.path(), 2, 2, None).unwrap();

        assert_eq!(
            first.entries.iter().map(|entry| entry.name.as_str()).collect::<Vec<_>>(),
            vec!["Z-folder", "a.txt"]
        );
        assert_eq!(first.total_entries, 4);
        assert_eq!(first.next_offset, 2);
        assert!(first.has_more);
        assert_eq!(
            second.entries.iter().map(|entry| entry.name.as_str()).collect::<Vec<_>>(),
            vec!["b.txt", "c.txt"]
        );
        assert_eq!(second.next_offset, 4);
        assert!(!second.has_more);
    }

    #[test]
    fn directory_query_filters_before_pagination() {
        let temp = TempDirectory::new();
        fs::write(temp.path().join("needle-a.txt"), b"a").unwrap();
        fs::write(temp.path().join("unrelated.txt"), b"x").unwrap();
        fs::write(temp.path().join("needle-b.txt"), b"b").unwrap();

        let first = super::read_directory_page(temp.path(), 0, 1, Some("needle")).unwrap();
        let second = super::read_directory_page(temp.path(), 1, 1, Some("needle")).unwrap();

        assert_eq!(first.total_entries, 2);
        assert_eq!(first.entries[0].name, "needle-a.txt");
        assert!(first.has_more);
        assert_eq!(second.entries[0].name, "needle-b.txt");
        assert!(!second.has_more);
    }

    #[test]
    fn directory_pages_keep_hidden_files_and_file_extensions() {
        let temp = TempDirectory::new();
        fs::write(temp.path().join(".private.txt"), b"secret").unwrap();
        fs::write(temp.path().join("notes.md"), b"notes").unwrap();

        let listing = super::read_directory_page(temp.path(), 0, 20, None).unwrap();
        let hidden = listing.entries.iter().find(|entry| entry.name == ".private.txt").unwrap();
        let visible = listing.entries.iter().find(|entry| entry.name == "notes.md").unwrap();

        assert!(hidden.is_hidden);
        assert_eq!(hidden.extension.as_deref(), Some("txt"));
        assert!(!visible.is_hidden);
        assert_eq!(visible.extension.as_deref(), Some("md"));
    }

    #[test]
    fn directory_page_limit_is_bounded() {
        assert_eq!(super::bounded_page_limit(None), 200);
        assert_eq!(super::bounded_page_limit(Some(1_000_000)), 500);
        assert_eq!(super::bounded_page_limit(Some(0)), 1);
    }
}
