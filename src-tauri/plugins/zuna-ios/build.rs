const COMMANDS: &[&str] = &["pick_folder", "restore_folder", "download_folder", "keep_awake", "share_file", "set_chrome"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).ios_path("ios").build();
}
