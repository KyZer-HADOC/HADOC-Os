#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn game_targets_accept_executables_and_windows_shortcuts_only() {
        assert!(super::is_supported_game_target(Path::new("C:/Games/Example.exe")));
        assert!(super::is_supported_game_target(Path::new("C:/Start Menu/Example.lnk")));
        assert!(super::is_supported_game_target(Path::new("C:/Start Menu/Example.url")));
        assert!(!super::is_supported_game_target(Path::new("C:/Games/setup.bat")));
        assert!(!super::is_supported_game_target(Path::new("C:/Games/readme.txt")));
    }

    #[test]
    fn shortcut_discovery_filters_for_known_game_and_launcher_names() {
        assert!(super::is_game_candidate("Minecraft Launcher"));
        assert!(super::is_game_candidate("Riot Client"));
        assert!(super::is_game_candidate("Epic Games Launcher"));
        assert!(!super::is_game_candidate("Windows Notepad"));
    }

    #[test]
    fn game_profile_parser_rejects_unrecognized_system_tweaks() {
        assert_eq!(super::parse_game_profile("balanced").unwrap().as_str(), "balanced");
        assert_eq!(super::parse_game_profile("gaming").unwrap().as_str(), "gaming");
        assert!(super::parse_game_profile("disable-defender").is_err());
        assert!(super::parse_game_profile("high-priority").is_err());
    }

    #[test]
    fn game_upsert_updates_a_saved_path_instead_of_duplicating_it() {
        let mut entries = Vec::new();
        let first = super::GameEntry::for_test(
            "Example",
            "C:/Games/example.exe",
            super::GameProfile::Balanced,
        );
        let updated = super::GameEntry::for_test(
            "Example Game",
            "C:/Games/example.exe",
            super::GameProfile::Gaming,
        );

        super::upsert_game(&mut entries, first);
        super::upsert_game(&mut entries, updated);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Example Game");
        assert_eq!(entries[0].profile.as_str(), "gaming");
    }
}
