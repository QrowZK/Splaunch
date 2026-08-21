mod customkey;
mod install;
mod launch;
mod maps;
mod scenario;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(launch::Game::default())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            launch::sp_locate_install,
            launch::sp_launch_preview,
            maps::sp_maps,
            scenario::spsc_script,
            scenario::spsc_problems,
            scenario::spsc_test,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Splaunch");
}
