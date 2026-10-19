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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default().plugin(tauri_plugin_dialog::init());
    // Mac: app restart + self-updater, exactly as before. The iPad gets updates from TestFlight instead.
    #[cfg(desktop)]
    let builder = builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build());
    // iPad: roast-folder picker (iCloud), keep screen on, share sheet.
    #[cfg(target_os = "ios")]
    let builder = builder.plugin(tauri_plugin_zuna_ios::init());
    builder
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
            roaster::ws_open,
            roaster::ws_send,
            roaster::ws_close,
            keep_awake,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Zuna Roast");
}
