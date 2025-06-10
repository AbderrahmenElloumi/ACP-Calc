// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod parser;

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
enum SectionContent {
    Matrix {
        columns: Vec<String>,
        index: Vec<String>,
        data: Vec<Vec<f64>>,
    },
    Messages(Vec<String>),
}

#[derive(Serialize, Deserialize)]
struct AcpOutput(pub HashMap<String, SectionContent>);

#[tauri::command]
async fn load_matrix_with_dialog(app_handle: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::sync::{Arc, Mutex};
    use tokio::sync::oneshot;
    
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));
    
    app_handle
        .dialog()
        .file()
        .add_filter("CSV files", &["csv"])
        .add_filter("Excel files", &["xlsx", "xls"])
        .add_filter("JSON files", &["json"])
        .add_filter("XML files", &["xml"])
        .add_filter("Text files", &["txt"])
        .add_filter("All files", &["*"])
        .pick_file(move |file_path| {
            if let Some(sender) = tx.lock().unwrap().take() {
                let _ = sender.send(file_path);
            }
        });
    
    match rx.await {
    Ok(Some(path)) => {
        if let Some(path_ref) = path.as_path() {
            let matrix = parser::parse_file(path_ref)?;
            serde_json::to_string(&matrix).map_err(|e| e.to_string())
        } else {
            Err("Invalid file path".to_string())
        }
    }
    Ok(None) => Err("No file selected".to_string()),
    Err(_) => Err("Dialog error".to_string()),
}
}

#[tauri::command]
fn acp(matrix: String, mut threshold: String) -> Result<AcpOutput, String> {
    let script_path = std::path::Path::new("python/process.py");
    if threshold.is_empty() {
        threshold = "0.925".to_string();
    }

    let output = std::process::Command::new("python")
        .arg(script_path)
        .arg(&matrix)
        .arg(threshold)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    let json_path = std::path::Path::new("output.json");
    let json_content = std::fs::read_to_string(json_path)
        .map_err(|e| format!("Failed to read output.json: {}", e))?;

    let structured: AcpOutput = serde_json::from_str(&json_content)
        .map_err(|e| format!("Failed to parse JSON output: {}", e))?;

    Ok(structured)
}

#[tauri::command]
fn load_matrix_file(path: String) -> Result<String, String> {
    let path = PathBuf::from(path);
    let matrix = parser::parse_file(&path)?;
    serde_json::to_string(&matrix).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![acp, load_matrix_file, load_matrix_with_dialog])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}