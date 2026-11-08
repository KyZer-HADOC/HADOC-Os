use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;

const MAX_DISCOVERY_DEPTH: usize = 5;
const MAX_DISCOVERY_ITEMS: usize = 1500;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GameProfile {
    Balanced,
    Gaming,
}

impl GameProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Balanced => "balanced",
            Self::Gaming => "gaming",
        }
    }
}

pub fn parse_game_profile(value: &str) -> Result<GameProfile, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "balanced" => Ok(GameProfile::Balanced),
        "gaming" => Ok(GameProfile::Gaming),
        _ => Err("Choose Balanced or Gaming / Low Resource.".into()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub profile: GameProfile,
}

impl GameEntry {
    fn from_path(name: String, path: PathBuf, profile: GameProfile) -> Self {
        let path = path.to_string_lossy().into_owned();
        let id = path.to_ascii_lowercase();
        Self { id, name, path, profile }
    }

    #[cfg(test)]
    fn for_test(name: &str, path: &str, profile: GameProfile) -> Self {
        Self::from_path(name.to_string(), PathBuf::from(path), profile)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GameCandidate {
    pub name: String,
    pub path: String,
}

pub struct GameLibraryState {
    entries: Mutex<Vec<GameEntry>>,
    store_path: PathBuf,
}

impl GameLibraryState {
    pub fn new(store_path: PathBuf) -> Result<Self, String> {
        if let Some(parent) = store_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Cannot create HADOC data folder: {error}"))?;
        }
        let entries = load_library(&store_path)?;
        Ok(Self {
            entries: Mutex::new(entries),
            store_path,
        })
    }

    fn persist(&self, entries: &[GameEntry]) -> Result<(), String> {
        let json = serde_json::to_vec_pretty(entries)
            .map_err(|error| format!("Cannot encode game library: {error}"))?;
        let temporary = self.store_path.with_extension("json.tmp");
        fs::write(&temporary, json)
            .map_err(|error| format!("Cannot save game library: {error}"))?;

        if self.store_path.exists() {
            fs::remove_file(&self.store_path)
                .map_err(|error| format!("Cannot replace saved game library: {error}"))?;
        }
        fs::rename(&temporary, &self.store_path)
            .map_err(|error| format!("Cannot finish saving game library: {error}"))
    }
}

fn load_library(path: &Path) -> Result<Vec<GameEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read(path)
        .map_err(|error| format!("Cannot read saved game library: {error}"))?;
    serde_json::from_slice(&content)
        .map_err(|error| format!("Saved game library is invalid JSON: {error}"))
}

pub fn is_supported_game_target(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(extension.to_ascii_lowercase().as_str(), "exe" | "lnk" | "url")
        })
}

pub fn is_game_candidate(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    const GAME_KEYWORDS: [&str; 22] = [
        "steam",
        "epic games",
        "riot client",
        "valorant",
        "league of legends",
        "minecraft",
        "roblox",
        "fortnite",
        "battle.net",
        "blizzard",
        "ea app",
        "ubisoft",
        "rockstar",
        "gog galaxy",
        "xbox",
        "genshin",
        "honkai",
        "counter-strike",
        "counter strike",
        "dota 2",
        "apex legends",
        "pubg",
    ];
    GAME_KEYWORDS.iter().any(|keyword| name.contains(keyword))
}

fn normalized_game_path(path: &str) -> String {
    path.replace('/', "\\").trim_end_matches('\\').to_ascii_lowercase()
}

fn upsert_game(entries: &mut Vec<GameEntry>, game: GameEntry) {
    let key = normalized_game_path(&game.path);
    if let Some(existing) = entries
        .iter_mut()
        .find(|existing| normalized_game_path(&existing.path) == key)
    {
        *existing = game;
    } else {
        entries.push(game);
    }
    entries.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
}

fn discover_from_root(root: &Path, candidates: &mut Vec<GameCandidate>, visited: &mut Vec<String>) {
    if candidates.len() >= MAX_DISCOVERY_ITEMS || !root.is_dir() {
        return;
    }

    let mut pending = vec![(root.to_path_buf(), 0usize)];
    while let Some((directory, depth)) = pending.pop() {
        if candidates.len() >= MAX_DISCOVERY_ITEMS {
            break;
        }
        let Ok(items) = fs::read_dir(&directory) else { continue };

        for item in items.flatten() {
            let path = item.path();
            let Ok(kind) = item.file_type() else { continue };
            if kind.is_dir() && depth < MAX_DISCOVERY_DEPTH {
                pending.push((path, depth + 1));
                continue;
            }
            if !kind.is_file() || !is_supported_game_target(&path) {
                continue;
            }

            let Some(name) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            let name = name.to_string();
            if !is_game_candidate(&name) {
                continue;
            }

            let resolved = path.canonicalize().unwrap_or(path);
            let path_text = resolved.to_string_lossy().into_owned();
            let key = normalized_game_path(&path_text);
            if visited.iter().any(|existing| existing == &key) {
                continue;
            }
            visited.push(key);
            candidates.push(GameCandidate {
                name,
                path: path_text,
            });
        }
    }
}

fn discover_game_shortcuts() -> Vec<GameCandidate> {
    let mut roots = Vec::new();
    if let Some(app_data) = std::env::var_os("APPDATA") {
        roots.push(PathBuf::from(app_data).join("Microsoft/Windows/Start Menu/Programs"));
    }
    if let Some(program_data) = std::env::var_os("ProgramData") {
        roots.push(PathBuf::from(program_data).join("Microsoft/Windows/Start Menu/Programs"));
    }

    let mut candidates = Vec::new();
    let mut visited = Vec::new();
    for root in roots {
        discover_from_root(&root, &mut candidates, &mut visited);
    }
    candidates.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    candidates
}

fn launch_windows_target(path: &Path) -> Result<(), String> {
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
        let target: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let directory = path
            .parent()
            .map(|parent| parent.as_os_str().encode_wide().chain(std::iter::once(0)).collect::<Vec<u16>>());
        let directory_pointer = directory.as_ref().map_or(ptr::null(), |value| value.as_ptr());
        let result = unsafe {
            ShellExecuteW(
                ptr::null_mut(),
                operation.as_ptr(),
                target.as_ptr(),
                ptr::null(),
                directory_pointer,
                1,
            )
        };

        if result as isize <= 32 {
            return Err(format!("Windows could not launch {}.", path.display()));
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = path;
        Err("Launching games is currently supported on Windows only.".into())
    }
}

#[tauri::command]
pub fn get_games(state: State<'_, GameLibraryState>) -> Result<Vec<GameEntry>, String> {
    state
        .entries
        .lock()
        .map(|entries| entries.clone())
        .map_err(|_| "Game library is temporarily unavailable.".into())
}

#[tauri::command]
pub fn add_game(
    name: String,
    path: String,
    profile: String,
    state: State<'_, GameLibraryState>,
) -> Result<Vec<GameEntry>, String> {
    let target = PathBuf::from(path.trim())
        .canonicalize()
        .map_err(|error| format!("Cannot find that game file: {error}"))?;
    if !target.is_file() || !is_supported_game_target(&target) {
        return Err("Choose a valid .exe, .lnk or .url game shortcut.".into());
    }

    let profile = parse_game_profile(&profile)?;
    let fallback_name = target
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Game")
        .to_string();
    let name = if name.trim().is_empty() {
        fallback_name
    } else {
        name.trim().to_string()
    };

    let mut entries = state
        .entries
        .lock()
        .map_err(|_| "Game library is temporarily unavailable.".to_string())?;
    upsert_game(&mut entries, GameEntry::from_path(name, target, profile));
    state.persist(&entries)?;
    Ok(entries.clone())
}

#[tauri::command]
pub fn set_game_profile(
    id: String,
    profile: String,
    state: State<'_, GameLibraryState>,
) -> Result<Vec<GameEntry>, String> {
    let profile = parse_game_profile(&profile)?;
    let mut entries = state
        .entries
        .lock()
        .map_err(|_| "Game library is temporarily unavailable.".to_string())?;
    let Some(game) = entries.iter_mut().find(|game| game.id == id) else {
        return Err("That game is no longer in your HADOC library.".into());
    };
    game.profile = profile;
    state.persist(&entries)?;
    Ok(entries.clone())
}

#[tauri::command]
pub fn remove_game(
    id: String,
    state: State<'_, GameLibraryState>,
) -> Result<Vec<GameEntry>, String> {
    let mut entries = state
        .entries
        .lock()
        .map_err(|_| "Game library is temporarily unavailable.".to_string())?;
    entries.retain(|game| game.id != id);
    state.persist(&entries)?;
    Ok(entries.clone())
}

#[tauri::command]
pub async fn discover_games() -> Vec<GameCandidate> {
    tauri::async_runtime::spawn_blocking(discover_game_shortcuts)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub fn launch_game(id: String, state: State<'_, GameLibraryState>) -> Result<String, String> {
    let game = {
        let entries = state
            .entries
            .lock()
            .map_err(|_| "Game library is temporarily unavailable.".to_string())?;
        entries
            .iter()
            .find(|game| game.id == id)
            .cloned()
            .ok_or_else(|| "That game is no longer in your HADOC library.".to_string())?
    };

    let path = PathBuf::from(&game.path);
    if !path.is_file() {
        return Err("The game file is missing. Remove it from HADOC and add its new location.".into());
    }
    if !is_supported_game_target(&path) {
        return Err("This game target type is not supported.".into());
    }
    launch_windows_target(&path)?;
    Ok(format!("Launching {}…", game.name))

    #[test]
    fn game_library_persists_profile_choices_between_loads() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let folder = std::env::temp_dir().join(format!("hadoc-game-library-{unique}"));
        let store_path = folder.join("games.json");
        let state = super::GameLibraryState::new(store_path.clone()).unwrap();
        let entry = super::GameEntry::for_test(
            "Test Game",
            "C:/Games/test-game.exe",
            super::GameProfile::Gaming,
        );

        state.persist(&[entry]).unwrap();
        drop(state);

        let reloaded = super::GameLibraryState::new(store_path).unwrap();
        let entries = reloaded.entries.lock().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Test Game");
        assert_eq!(entries[0].profile.as_str(), "gaming");
        drop(entries);
        drop(reloaded);
        fs::remove_dir_all(folder).unwrap();
    }
}
