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
}
