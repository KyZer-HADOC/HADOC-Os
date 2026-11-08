mod file_manager;
mod system_info;

use std::process::Command;
use tauri::Manager;

#[tauri::command]
fn get_platform_info() -> String {
    "Windows 10 / 11".to_string()
}

#[tauri::command]
fn launch_windows_app(app: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let (program, args): (&str, &[&str]) = match app.as_str() {
            "explorer" => ("explorer.exe", &[]),
            "settings" => ("cmd", &["/C", "start", "", "ms-settings:"]),
            "terminal" => ("cmd", &["/C", "start", "", "wt.exe"]),
            "taskmgr" => ("taskmgr.exe", &[]),
            "notepad" => ("notepad.exe", &[]),
            "browser" => ("cmd", &["/C", "start", "", "https://www.google.com"]),
            _ => return Err("This shortcut is not available yet.".into()),
        };
        Command::new(program).args(args).spawn()
            .map(|_| format!("Launched {app}"))
            .map_err(|e| format!("Could not launch {app}: {e}"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
        Err("HADOC OS currently targets Windows.".into())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_platform_info, launch_windows_app, file_manager::list_directory, file_manager::get_quick_locations, file_manager::open_path, system_info::get_system_snapshot])
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.set_title("HADOC OS — Preview")?;
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running HADOC OS");
}
