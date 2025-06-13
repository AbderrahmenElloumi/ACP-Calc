// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod parser;

use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use umya_spreadsheet::{self};
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};
use quick_xml::Writer as XmlWriter;
use serde_json;

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
fn acp(matrix: String, threshold: String) -> Result<AcpOutput, String> {
    let script_path = std::path::Path::new("python/process.py");
    
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

#[tauri::command]
fn export_matrix(acpresult: String, format: String, whichmatrix: String, fne: String) -> Result<String, String> {
    let all_matrices: AcpOutput = serde_json::from_str(&acpresult)
        .map_err(|e| format!("Failed to parse matrix JSON: {}", e))?;

    let selected_section = all_matrices.0.get(&whichmatrix)
        .ok_or_else(|| format!("Matrix '{}' not found in input", whichmatrix))?;

    let filename = format!("{}.{}", fne, format.to_lowercase());

    match selected_section {
        SectionContent::Matrix { columns:_, index:_, data } => {
            match format.to_lowercase().as_str() {
                "csv" => {
                    let mut wtr = csv::Writer::from_path(&filename).map_err(|e| e.to_string())?;
                    for row in data {
                        wtr.serialize(row).map_err(|e| e.to_string())?;
                    }
                    wtr.flush().map_err(|e| e.to_string())?;
                }

                "json" => {
                    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
                    std::fs::write(&filename, json).map_err(|e| e.to_string())?;
                }

                "txt" => {
                    let mut file = File::create(&filename).map_err(|e| e.to_string())?;
                    for row in data {
                        let line: Vec<String> = row.iter().map(|v| v.to_string()).collect();
                        writeln!(file, "{}", line.join("\t")).map_err(|e| e.to_string())?;
                    }
                }

                "xml" => {
                    let file = File::create(&filename).map_err(|e| e.to_string())?;
                    let mut writer = XmlWriter::new_with_indent(file, b' ', 2);

                    writer.write_event(Event::Start(BytesStart::new("matrix")))
                        .map_err(|e| e.to_string())?;

                    for row in data {
                        writer.write_event(Event::Start(BytesStart::new("row"))).map_err(|e| e.to_string())?;

                        for val in row {
                            writer.write_event(Event::Start(BytesStart::new("cell"))).map_err(|e| e.to_string())?;
                            writer.write_event(Event::Text(BytesText::from_escaped(val.to_string())))
                                .map_err(|e| e.to_string())?;
                            writer.write_event(Event::End(BytesEnd::new("cell"))).map_err(|e| e.to_string())?;
                        }

                        writer.write_event(Event::End(BytesEnd::new("row"))).map_err(|e| e.to_string())?;
                    }

                    writer.write_event(Event::End(BytesEnd::new("matrix"))).map_err(|e| e.to_string())?;
                }

                "xlsx" | "xlsm" | "ods" | "xls" => {
                    let mut book = umya_spreadsheet::new_file();
                    let sheet = book.get_sheet_by_name_mut("Sheet1").unwrap();

                    for (i, row) in data.iter().enumerate() {
                        for (j, val) in row.iter().enumerate() {
                            let col_letter = ((b'A' + j as u8) as char).to_string();
                            let cell_ref = format!("{}{}", col_letter, i + 1);
                            sheet.get_cell_mut(cell_ref.as_str()).set_value(val.to_string());
                        }
                    }

                    umya_spreadsheet::writer::xlsx::write(&book, &filename).map_err(|e| e.to_string())?;
                }

                _ => return Err(format!("Unsupported export format: {}", format)),
            }
        }

        SectionContent::Messages(_) => {
            return Err(format!(
                "'{}' is not a matrix and cannot be exported in this format.",
                whichmatrix
            ));
        }
    }

    Ok(filename)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![acp, 
                                                load_matrix_file, 
                                                load_matrix_with_dialog, 
                                                export_matrix])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}