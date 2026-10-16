//! Roast data on disk.
//!
//! Everything lives in one data folder (default: ~/Documents/Zuna Roast).
//! The web side only ever passes paths *relative* to that folder, and they are
//! checked so nothing outside it can be read or written. Writes go to a
//! temporary file first and are then renamed into place, so a crash or power
//! cut mid-save never leaves a half-written roast behind.

use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

pub struct DataFolder(pub Mutex<Option<PathBuf>>);

const SETTING_FILE: &str = "data-folder.txt";
const DEFAULT_NAME: &str = "Zuna Roast";

fn setting_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(SETTING_FILE))
}

fn default_folder(app: &AppHandle) -> Result<PathBuf, String> {
    let docs = app.path().document_dir().map_err(|e| e.to_string())?;
    Ok(docs.join(DEFAULT_NAME))
}

/// The data folder, created if it does not exist yet.
fn root(app: &AppHandle, state: &State<DataFolder>) -> Result<PathBuf, String> {
    let mut guard = state.0.lock().map_err(|_| "lock".to_string())?;
    if let Some(p) = guard.as_ref() {
        return Ok(p.clone());
    }
    let chosen = fs::read_to_string(setting_path(app)?)
        .ok()
        .map(|s| PathBuf::from(s.trim()))
        .filter(|p| p.is_absolute());
    let p = match chosen {
        Some(p) => p,
        None => default_folder(app)?,
    };
    fs::create_dir_all(&p).map_err(|e| format!("Could not create {}: {}", p.display(), e))?;
    *guard = Some(p.clone());
    Ok(p)
}

/// Join a relative path onto the data folder, refusing anything that could
/// escape it ("..", absolute paths, drive prefixes).
fn resolve(base: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_path = Path::new(rel);
    for c in rel_path.components() {
        match c {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => return Err(format!("Not allowed: {}", rel)),
        }
    }
    Ok(base.join(rel_path))
}

fn not_found(p: &Path) -> String {
    format!("NotFound: {}", p.display())
}

#[derive(Serialize)]
pub struct Entry {
    name: String,
    dir: bool,
}

#[tauri::command]
pub fn data_folder(app: AppHandle, state: State<DataFolder>) -> Result<String, String> {
    Ok(root(&app, &state)?.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn set_data_folder(app: AppHandle, state: State<DataFolder>, path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    if !p.is_absolute() {
        return Err("Choose a folder".into());
    }
    fs::create_dir_all(&p).map_err(|e| e.to_string())?;
    fs::write(setting_path(&app)?, p.to_string_lossy().as_bytes()).map_err(|e| e.to_string())?;
    *state.0.lock().map_err(|_| "lock".to_string())? = Some(p.clone());
    Ok(p.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn fs_read(app: AppHandle, state: State<DataFolder>, rel: String) -> Result<String, String> {
    let p = resolve(&root(&app, &state)?, &rel)?;
    if !p.is_file() {
        return Err(not_found(&p));
    }
    fs::read_to_string(&p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn fs_write(app: AppHandle, state: State<DataFolder>, rel: String, text: String) -> Result<(), String> {
    let p = resolve(&root(&app, &state)?, &rel)?;
    write_atomic(&p, text.as_bytes())
}

pub fn write_atomic(p: &Path, bytes: &[u8]) -> Result<(), String> {
    let dir = p.parent().ok_or("Bad path")?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let name = p.file_name().ok_or("Bad path")?.to_string_lossy();
    let tmp = dir.join(format!(".{}.saving", name));
    {
        let mut f = fs::File::create(&tmp).map_err(|e| e.to_string())?;
        f.write_all(bytes).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
    }
    fs::rename(&tmp, p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn fs_list(app: AppHandle, state: State<DataFolder>, rel: String) -> Result<Vec<Entry>, String> {
    let p = resolve(&root(&app, &state)?, &rel)?;
    if !p.is_dir() {
        return Err(not_found(&p));
    }
    let mut out = Vec::new();
    for e in fs::read_dir(&p).map_err(|e| e.to_string())? {
        let e = e.map_err(|e| e.to_string())?;
        let name = e.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue; // skip hidden files and unfinished saves
        }
        let dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        out.push(Entry { name, dir });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

#[tauri::command]
pub fn fs_is_dir(app: AppHandle, state: State<DataFolder>, rel: String) -> Result<bool, String> {
    Ok(resolve(&root(&app, &state)?, &rel)?.is_dir())
}

#[tauri::command]
pub fn fs_mkdir(app: AppHandle, state: State<DataFolder>, rel: String) -> Result<(), String> {
    let p = resolve(&root(&app, &state)?, &rel)?;
    fs::create_dir_all(&p).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn fs_remove(app: AppHandle, state: State<DataFolder>, rel: String) -> Result<(), String> {
    let p = resolve(&root(&app, &state)?, &rel)?;
    if p.is_file() {
        fs::remove_file(&p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Save an export (CSV / JSON) to a place the person picked in a Save dialog.
#[tauri::command]
pub fn save_export(path: String, text: String) -> Result<(), String> {
    let p = PathBuf::from(path);
    if !p.is_absolute() {
        return Err("Choose where to save".into());
    }
    write_atomic(&p, text.as_bytes())
}

/// Show the data folder in Finder.
#[tauri::command]
pub fn reveal_data_folder(app: AppHandle, state: State<DataFolder>) -> Result<(), String> {
    let p = root(&app, &state)?;
    std::process::Command::new("open")
        .arg(&p)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_escape() {
        let b = Path::new("/data");
        assert!(resolve(b, "../x").is_err());
        assert!(resolve(b, "/etc/passwd").is_err());
        assert!(resolve(b, "Roasts/../../x").is_err());
        assert_eq!(resolve(b, "Roasts/2026/a.json").unwrap(), PathBuf::from("/data/Roasts/2026/a.json"));
    }

    #[test]
    fn atomic_write_replaces() {
        let dir = std::env::temp_dir().join(format!("zr-test-{}", std::process::id()));
        let f = dir.join("sub/a.json");
        write_atomic(&f, b"one").unwrap();
        write_atomic(&f, b"two").unwrap();
        assert_eq!(fs::read_to_string(&f).unwrap(), "two");
        assert!(!dir.join("sub/.a.json.saving").exists());
        fs::remove_dir_all(dir).ok();
    }
}
