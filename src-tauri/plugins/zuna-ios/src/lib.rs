//! iPad-only helpers for Zuna Roast. Only compiled into the iPad app (see the
//! `target_os = "ios"` dependency in src-tauri/Cargo.toml) — the Mac app never includes it.
//!
//! * pick_folder / restore_folder — the iPad can't open a folder by path the way the Mac
//!   does. It shows the Files picker once (pick iCloud Drive › Desktop › ZUNA COFFEE › Zuna Roast),
//!   remembers that choice, and re-opens it every time the app starts.
//! * download_folder — makes sure every roast file in that iCloud folder is actually on
//!   the iPad (iCloud can leave files "in the cloud" until they're asked for).
//! * keep_awake — keeps the screen on during a roast.
//! * share_file — exports (CSV etc.) go out through the iPad share sheet.
//! * set_chrome — status bar and screen edges follow the app theme (no white strips).

use serde::Serialize;
use serde_json::Value;
use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    AppHandle, Manager, Runtime,
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_zuna_ios);

struct ZunaIos<R: Runtime>(PluginHandle<R>);

#[derive(Serialize)]
struct Awake {
    on: bool,
}

#[derive(Serialize)]
struct Chrome {
    color: String,
    dark: bool,
}

#[derive(Serialize)]
struct Share {
    name: String,
    text: String,
}

/// Run a native call off the main thread (the picker and share sheet wait for the user).
async fn native<R: Runtime, P: Serialize + Send + 'static>(app: AppHandle<R>, method: &'static str, payload: P) -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let h = app.state::<ZunaIos<R>>();
        h.0.run_mobile_plugin::<Value>(method, payload).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn pick_folder<R: Runtime>(app: AppHandle<R>) -> Result<Value, String> {
    native(app, "pickFolder", ()).await
}

#[tauri::command]
async fn restore_folder<R: Runtime>(app: AppHandle<R>) -> Result<Value, String> {
    native(app, "restoreFolder", ()).await
}

#[tauri::command]
async fn download_folder<R: Runtime>(app: AppHandle<R>) -> Result<Value, String> {
    native(app, "downloadFolder", ()).await
}

#[tauri::command]
async fn keep_awake<R: Runtime>(app: AppHandle<R>, on: bool) -> Result<Value, String> {
    native(app, "keepAwake", Awake { on }).await
}

#[tauri::command]
async fn share_file<R: Runtime>(app: AppHandle<R>, name: String, text: String) -> Result<Value, String> {
    native(app, "shareFile", Share { name, text }).await
}

/// Paint the screen edges (status bar strip, home-indicator strip) in the app's background
/// color, and pick light or dark status-bar text to match.
#[tauri::command]
async fn set_chrome<R: Runtime>(app: AppHandle<R>, color: String, dark: bool) -> Result<Value, String> {
    native(app, "setChrome", Chrome { color, dark }).await
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("zuna-ios")
        .invoke_handler(tauri::generate_handler![pick_folder, restore_folder, download_folder, keep_awake, share_file, set_chrome])
        .setup(|app, api| {
            #[cfg(target_os = "ios")]
            {
                let handle = api.register_ios_plugin(init_plugin_zuna_ios)?;
                app.manage(ZunaIos(handle));
            }
            #[cfg(not(target_os = "ios"))]
            {
                let _ = (app, api);
            }
            Ok(())
        })
        .build()
}
