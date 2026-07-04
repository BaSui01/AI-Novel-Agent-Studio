use gateway::start_gateway_server;
use novel_core::{DatabaseManager, Project};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

pub struct AppState {
    pub db: Arc<DatabaseManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPromptItem {
    pub id: String,
    pub role_name: String,
    pub display_name: String,
    pub system_prompt: String,
    pub user_template: String,
    pub is_default: bool,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! Welcome to AI Novel Agent Studio Gateway.", name)
}

#[tauri::command]
fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    state.db.get_projects().map_err(|e| e.to_string())
}

#[tauri::command]
fn create_project(state: State<'_, AppState>, project: Project) -> Result<(), String> {
    state.db.create_project(&project).map_err(|e| e.to_string())
}

#[tauri::command]
fn update_chapter(
    state: State<'_, AppState>,
    chapter_id: String,
    content: String,
    word_count: usize,
) -> Result<(), String> {
    state
        .db
        .update_chapter_content(&chapter_id, &content, word_count)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize SQLite in memory for desktop app initialization
    let db = DatabaseManager::in_memory().expect("Failed to initialize SQLite database");
    let db_arc = Arc::new(db);

    // Spawn AI Gateway on port 8080 using Tauri async runtime
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_gateway_server(8080).await {
            eprintln!("AI Gateway server error: {}", e);
        }
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState { db: db_arc })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_projects,
            create_project,
            update_chapter
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
