// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

mod commands;
use dotenv;
use rig::providers::gemini;
use rig::vector_store::in_memory_store::InMemoryVectorStore;
use tauri::{
    async_runtime::Mutex,
    Manager,
};

struct AppState {
    vector_store: Mutex<InMemoryVectorStore<String>>,
    client: gemini::Client,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            dotenv::dotenv().ok();
            let vc: InMemoryVectorStore<String> = InMemoryVectorStore::default();
            let client = gemini::Client::from_env();

            let state = AppState {
                vector_store: Mutex::new(vc),
                client,
            };

            app.manage(state);

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![commands::prompt, commands::index_folders])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
