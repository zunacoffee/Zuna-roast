mod files;
mod roaster;

use std::process::{Child, Command};
use std::sync::Mutex;
use tauri::State;

/// Keeps the Mac (and its display) awake while a roast is recording or Control is on,
/// using macOS's built-in `caffeinate`. It stops by itself if the app quits.
#[derive(Default)]
struct Awake(Mutex<Option<Child>>);

#[tauri::command]
fn keep_awake(state: State<Awake>, on: bool) -> Result<(), String> {
    let mut g = state.0.lock().map_err(|_| "lock")?;
    if on {
        if g.is_none() {
            let child = Command::new("caffeinate")
                .args(["-d", "-i", "-w", &std::process::id().to_string()])
                .spawn()
                .map_err(|e| e.to_string())?;
            *g = Some(child);
        }
    } else if let Some(mut c) = g.take() {
        let _ = c.kill();
        let _ = c.wait();
    }
    Ok(())
}

/// A friendly name for this device, shown when another device asks to take over Control
/// ("Control is on at Ty's iMac"). On a Mac it's the Computer Name from System Settings › General › Sharing.
#[tauri::command]
fn device_name() -> String {
    #[cfg(target_os = "macos")]
    let name = Command::new("scutil")
        .args(["--get", "ComputerName"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Mac".into());
    #[cfg(target_os = "ios")]
    let name = String::from("iPad");
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    let name = String::from("This computer");
    name
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(files::DataFolder(Mutex::new(None)))
        .manage(roaster::Sockets::default())
        .manage(Awake::default())
        .invoke_handler(tauri::generate_handler![
            files::data_folder,
            files::set_data_folder,
            files::fs_read,
            files::fs_write,
            files::fs_list,
            files::fs_is_dir,
            files::fs_mkdir,
            files::fs_remove,
            files::save_export,
            files::reveal_data_folder,
            files::fs_stamp,
            device_name,
            roaster::ws_open,
            roaster::ws_send,
            roaster::ws_close,
            keep_awake,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Zuna Roast");
}
