use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub const THIS_PC_PATH: &str = "__HADOC_THIS_PC__";
const DEFAULT_PAGE_SIZE: usize = 200;
const MAX_PAGE_SIZE: usize = 500;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_hidden: bool,
    pub size: Option<u64>,
    pub modified: Option<u64>,
    pub extension: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DirectoryListing {
    pub path: String,
    pub parent: Option<String>,
    pub entries: Vec<FileEntry>,
    pub total_entries: usize,
    pub next_offset: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuickLocation {
    pub name: String,
    pub path: String,
    pub icon: String,
    pub group: String,
}

#[derive(Debug)]
struct DirectoryItem {
    name: String,
    path: PathBuf,
    is_dir: bool,
}

pub fn bounded_page_limit(limit: Option<usize>) -> usize {
    limit.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE)
}

fn normalized_location_path(path: &str) -> String {
    path.replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

fn push_existing_location(
    locations: &mut Vec<QuickLocation>,
    name: &str,
    path: &Path,
    icon: &str,
    group: &str,
) {
    if !path.is_dir() {
        return;
    }

    let resolved = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let display_path = resolved.to_string_lossy().into_owned();
    let key = normalized_location_path(&display_path);

    if locations
        .iter()
        .any(|location| normalized_location_path(&location.path) == key)
    {
        return;
    }

    locations.push(QuickLocation {
        name: name.to_string(),
        path: display_path,
        icon: icon.to_string(),
        group: group.to_string(),
    });
}

fn push_environment_location(
    locations: &mut Vec<QuickLocation>,
    variable: &str,
    name: &str,
    icon: &str,
    group: &str,
) {
    if let Some(path) = std::env::var_os(variable) {
        push_existing_location(locations, name, &PathBuf::from(path), icon, group);
    }
}

fn home_directory() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

fn current_logical_drive_mask() -> u32 {
    #[cfg(windows)]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetLogicalDrives() -> u32;
        }

        // GetLogicalDrives returns a bit for each currently assigned drive
        // letter. It avoids probing every possible letter (which can block on
        // stale mapped network drives).
        unsafe { GetLogicalDrives() }
    }

    #[cfg(not(windows))]
    {
        0
    }
}

fn logical_drive_roots(mask: u32) -> Vec<PathBuf> {
    (0..26)
        .filter(|index| mask & (1u32 << index) != 0)
        .map(|index| PathBuf::from(format!("{}:\\", (b'A' + index as u8) as char)))
        .collect()
}

fn drive_entry(root: &Path) -> FileEntry {
    let path = root.to_string_lossy().into_owned();
    let trimmed = path.trim_end_matches('\\').trim_end_matches('/');
    let name = trimmed
        .rsplit(|character| character == '\\' || character == '/')
        .next()
        .unwrap_or(trimmed)
        .to_string();

    FileEntry {
        name,
        path,
        is_dir: true,
        is_hidden: false,
        size: None,
        modified: None,
        extension: None,
    }
}

fn this_pc_listing_from_roots(roots: &[PathBuf]) -> DirectoryListing {
    let mut entries = roots.iter().map(|root| drive_entry(root)).collect::<Vec<_>>();
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    let total_entries = entries.len();

    DirectoryListing {
        path: THIS_PC_PATH.to_string(),
        parent: None,
        entries,
        total_entries,
        next_offset: total_entries,
        has_more: false,
    }
}

fn page_virtual_listing(
    mut listing: DirectoryListing,
    offset: usize,
    limit: usize,
    query: Option<&str>,
) -> DirectoryListing {
    if let Some(query) = query.map(str::trim).filter(|query| !query.is_empty()) {
        let needle = query.to_lowercase();
        listing
            .entries
            .retain(|entry| entry.name.to_lowercase().contains(&needle));
    }

    let total_entries = listing.entries.len();
    let start = offset.min(total_entries);
    let end = start.saturating_add(limit).min(total_entries);
    let entries = listing.entries.drain(start..end).collect();

    DirectoryListing {
        path: listing.path,
        parent: listing.parent,
        entries,
        total_entries,
        next_offset: end,
        has_more: end < total_entries,
    }
}

fn this_pc_listing() -> DirectoryListing {
    this_pc_listing_from_roots(&logical_drive_roots(current_logical_drive_mask()))
}

fn get_quick_locations_sync() -> Vec<QuickLocation> {
    let mut locations = vec![QuickLocation {
        name: "This PC".to_string(),
        path: THIS_PC_PATH.to_string(),
        icon: "▰".to_string(),
        group: "Computer".to_string(),
    }];

    if let Some(home) = home_directory() {
        push_existing_location(&mut locations, "Home", &home, "⌂", "User folders");

        let folders = [
            ("Desktop", "Desktop", "▣"),
            ("Documents", "Documents", "▤"),
            ("Downloads", "Downloads", "⇩"),
            ("Pictures", "Pictures", "▧"),
            ("Music", "Music", "♫"),
            ("Videos", "Videos", "▶"),
            ("Saved Games", "Saved Games", "◇"),
            ("Links", "Links", "↗"),
            ("Contacts", "Contacts", "♙"),
            ("Searches", "Searches", "⌕"),
        ];

        for (name, folder, icon) in folders {
            push_existing_location(
                &mut locations,
                name,
                &home.join(folder),
                icon,
                "User folders",
            );
        }
    }

    let environment_locations = [
        ("OneDrive", "OneDrive", "☁", "Cloud"),
        ("OneDriveCommercial", "OneDrive (Work)", "☁", "Cloud"),
        ("OneDriveConsumer", "OneDrive (Personal)", "☁", "Cloud"),
        ("APPDATA", "AppData · Roaming", "⚙", "Windows folders"),
        ("LOCALAPPDATA", "AppData · Local", "⚙", "Windows folders"),
        ("TEMP", "Temporary files", "◷", "Windows folders"),
        ("PUBLIC", "Public", "♧", "Windows folders"),
        ("ProgramFiles", "Program Files", "▤", "Windows folders"),
        ("ProgramW6432", "Program Files (64-bit)", "▤", "Windows folders"),
        ("ProgramFiles(x86)", "Program Files (x86)", "▤", "Windows folders"),
        ("ProgramData", "ProgramData", "▤", "Windows folders"),
        ("windir", "Windows", "▣", "Windows folders"),
        ("SystemRoot", "System Root", "▣", "Windows folders"),
    ];

    for (variable, name, icon, group) in environment_locations {
        push_environment_location(&mut locations, variable, name, icon, group);
    }

    if let Some(windows) = std::env::var_os("windir").or_else(|| std::env::var_os("SystemRoot")) {
        push_existing_location(
            &mut locations,
            "System32",
            &PathBuf::from(windows).join("System32"),
            "▣",
            "Windows folders",
        );
    }

    for root in logical_drive_roots(current_logical_drive_mask()) {
        let drive = drive_entry(&root);
        let key = normalized_location_path(&drive.path);
        if !locations
            .iter()
            .any(|location| normalized_location_path(&location.path) == key)
        {
            locations.push(QuickLocation {
                name: drive.name.clone(),
                path: drive.path,
                icon: "▰".to_string(),
                group: "Drives".to_string(),
            });
        }
    }

    locations
}

fn is_hidden_name(name: &str, windows_attributes: Option<u32>) -> bool {
    name.starts_with('.')
        || windows_attributes.is_some_and(|attributes| attributes & 0x2 != 0)
}

fn windows_attributes(metadata: Option<&fs::Metadata>) -> Option<u32> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.map(MetadataExt::file_attributes)
    }

    #[cfg(not(windows))]
    {
        let _ = metadata;
        None
    }
}

fn parent_for_directory(path: &Path) -> Option<String> {
    if let Some(parent) = path.parent() {
        if parent != path {
            return Some(parent.to_string_lossy().into_owned());
        }
    }

    #[cfg(windows)]
    if path.has_root() && path.components().count() == 2 {
        return Some(THIS_PC_PATH.to_string());
    }

    None
}

fn read_directory_page(
    path: &Path,
    offset: usize,
    limit: usize,
    query: Option<&str>,
) -> Result<DirectoryListing, String> {
    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Cannot access {}: {error}", path.display()))?;

    if !canonical.is_dir() {
        return Err(format!("{} is not a folder.", canonical.display()));
    }

    let needle = query
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase);

    // Read entry names and directory flags first. File size and timestamps are
    // queried only for the page the UI is about to display.
    let mut items = Vec::new();
    let directory = fs::read_dir(&canonical)
        .map_err(|error| format!("Cannot read {}: {error}", canonical.display()))?;

    for entry in directory {
        let Ok(entry) = entry else { continue };
        let entry_path = entry.path();
        let Ok(file_type) = entry.file_type() else { continue };
        let is_dir = file_type.is_dir() || (file_type.is_symlink() && entry_path.is_dir());
        let name = entry.file_name().to_string_lossy().into_owned();

        if needle
            .as_ref()
            .is_some_and(|needle| !name.to_lowercase().contains(needle))
        {
            continue;
        }

        items.push(DirectoryItem {
            name,
            path: entry_path,
            is_dir,
        });
    }

    items.sort_by(|left, right| {
        right
            .is_dir
            .cmp(&left.is_dir)
            .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
            .then_with(|| left.name.cmp(&right.name))
    });

    let total_entries = items.len();
    let start = offset.min(total_entries);
    let end = start.saturating_add(limit).min(total_entries);
    let mut entries = Vec::with_capacity(end.saturating_sub(start));

    for item in items.drain(start..end) {
        let metadata = fs::metadata(&item.path).ok();
        let hidden = is_hidden_name(&item.name, windows_attributes(metadata.as_ref()));
        let modified = metadata
            .as_ref()
            .and_then(|metadata| metadata.modified().ok())
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs());
        let size = metadata
            .as_ref()
            .filter(|metadata| metadata.is_file())
            .map(fs::Metadata::len);
        let extension = Path::new(&item.name)
            .extension()
            .map(|extension| extension.to_string_lossy().to_ascii_lowercase());

        entries.push(FileEntry {
            name: item.name,
            path: item.path.to_string_lossy().into_owned(),
            is_dir: item.is_dir,
            is_hidden: hidden,
            size,
            modified,
            extension,
        });
    }

    Ok(DirectoryListing {
        path: canonical.to_string_lossy().into_owned(),
        parent: parent_for_directory(&canonical),
        entries,
        total_entries,
        next_offset: end,
        has_more: end < total_entries,
    })
}

#[tauri::command]
pub async fn list_directory(
    path: Option<String>,
    offset: Option<usize>,
    limit: Option<usize>,
    query: Option<String>,
) -> Result<DirectoryListing, String> {
    let offset = offset.unwrap_or(0);
    let limit = bounded_page_limit(limit);

    tauri::async_runtime::spawn_blocking(move || {
        if path.as_deref().is_none_or(|path| path == THIS_PC_PATH) {
            return Ok(page_virtual_listing(
                this_pc_listing(),
                offset,
                limit,
                query.as_deref(),
            ));
        }

        let Some(path) = path else {
            return Ok(this_pc_listing());
        };
        if path.trim().is_empty() {
            return Err("Enter a folder path.".to_string());
        }

        read_directory_page(Path::new(&path), offset, limit, query.as_deref())
    })
    .await
    .map_err(|error| format!("File Manager operation failed: {error}"))?
}

#[tauri::command]
pub async fn get_quick_locations() -> Vec<QuickLocation> {
    tauri::async_runtime::spawn_blocking(get_quick_locations_sync)
        .await
        .unwrap_or_default()
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
        let target_wide: Vec<u16> = target
            .as_os_str()
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

#[cfg(test)]
mod tests {
    use super::read_directory_page;
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

        let listing = read_directory_page(temp.path(), 0, 100, None).unwrap();

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

        assert!(read_directory_page(&file, 0, 100, None).is_err());
    }

    #[test]
    fn read_directory_returns_an_empty_list_for_an_empty_folder() {
        let temp = TempDirectory::new();

        let listing = read_directory_page(temp.path(), 0, 100, None).unwrap();

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

    #[cfg(windows)]
    #[test]
    fn drive_root_parent_is_this_pc() {
        assert_eq!(
            super::parent_for_directory(Path::new(r"C:\\")),
            Some(super::THIS_PC_PATH.to_string())
        );
    }
}
