// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![acp])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
