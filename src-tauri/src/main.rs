#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod github;
mod keychain;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(github::Gh::default())
        .invoke_handler(tauri::generate_handler![
            config::load_config,
            github::session_connect,
            github::session_connect_token,
            github::gh_load_tree,
            github::gh_read_file,
            github::gh_file_sha,
            github::gh_create_branch,
            github::gh_save_file,
            github::gh_create_pr,
            github::gh_delete_branch,
            github::gh_find_stale_sessions
        ])
        .run(tauri::generate_context!())
        .expect("error while running Docs Editor");
}
