// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::process::Command;

#[tauri::command]
fn acp(matrix: String, threshold: String) -> Result<String, String> {
    let script_path = std::path::Path::new("python/process.py");
    if threshold.is_empty() {
        threshold = "0.925".to_string();
    }
    let output = Command::new("python")
        .arg(script_path)
        .arg(matrix)
        .arg(threshold)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        // Read the output.json file
        let json_path = std::path::Path::new("python/output.json");
        let json_content = fs::read_to_string(json_path)
            .map_err(|e| format!("Failed to read output.json: {}", e))?;

        Ok(json_content)  // Return JSON string to frontend
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, acp])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
