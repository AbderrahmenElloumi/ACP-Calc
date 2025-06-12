use calamine::{open_workbook_auto, Reader, DataType};
use csv::ReaderBuilder;
use quick_xml::events::Event;
use quick_xml::reader::Reader as XmlReader;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn parse_file(path: &Path) -> Result<Vec<Vec<f64>>, String> {
    match path.extension().and_then(|s| s.to_str()) {
        Some("csv") => parse_csv(path),
        Some("xlsx") | Some("xls") | Some("ods") | Some("xlsm") => parse_excel_with_serde(path),
        Some("json") => parse_json(path),
        Some("xml") => parse_xml(path),
        Some("txt") => parse_txt(path),
        _ => Err("Unsupported file format".to_string()),
    }
}

fn parse_csv(path: &Path) -> Result<Vec<Vec<f64>>, String> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)
        .map_err(|e| e.to_string())?;

    let mut matrix = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| e.to_string())?;
        let row: Result<Vec<f64>, _> = record
            .iter()
            .map(|s| s.parse::<f64>())
            .collect();
        matrix.push(row.map_err(|e| e.to_string())?);
    }
    Ok(matrix)
}

pub fn parse_excel_with_serde<Row>(path: &Path) -> Result<Vec<Row>, String> 
where
    Row: serde::de::DeserializeOwned,
{
    let mut workbook = open_workbook_auto(path).map_err(|e| e.to_string())?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or("No sheets found")?
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();

    for (row_idx, row) in range.rows().enumerate() {
        let stringified: Result<Vec<String>, String> = row.iter().enumerate().map(|(col_idx, cell)| {
            match cell {
                DataType::Float(f) => Ok(f.to_string()),
                DataType::Int(i) => Ok(i.to_string()),
                DataType::String(s) => {
                    // Try to parse string as number first
                    if let Ok(num) = s.parse::<f64>() {
                        Ok(num.to_string())
                    } else {
                        Err(format!(
                            "Non-numeric string '{}' found at row {}, column {} - expected numeric matrix", 
                            s, row_idx + 1, col_idx + 1
                        ))
                    }
                },
                DataType::Empty => Ok("0".to_string()), // You might want to make this configurable
                DataType::Bool(b) => Err(format!(
                    "Boolean value '{}' found at row {}, column {} - expected numeric matrix", 
                    b, row_idx + 1, col_idx + 1
                )),
                DataType::Error(e) => Err(format!(
                    "Excel error '{:?}' found at row {}, column {} - cannot parse", 
                    e, row_idx + 1, col_idx + 1
                )),
                DataType::DateTime(dt) => Err(format!(
                    "DateTime value '{:?}' found at row {}, column {} - expected numeric matrix", 
                    dt, row_idx + 1, col_idx + 1
                )),
                DataType::DateTimeIso(dt) => Err(format!(
                    "DateTimeIso value '{}' found at row {}, column {} - expected numeric matrix", 
                    dt, row_idx + 1, col_idx + 1
                )),
                DataType::DurationIso(dur) => Err(format!(
                    "DurationIso value '{}' found at row {}, column {} - expected numeric matrix", 
                    dur, row_idx + 1, col_idx + 1
                )),
                DataType::Duration(dur) => Err(format!(
                    "Duration value '{}' found at row {}, column {} - expected numeric matrix", 
                    dur, row_idx + 1, col_idx + 1
                )),
            }
        }).collect();

        let stringified = stringified?;

        // Deserialize from a CSV-like row
        let csv_line = stringified.join(",");

        let parsed: Row = match csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_line.as_bytes())
            .deserialize()
            .next()
        {
            Some(Ok(row)) => row,
            Some(Err(e)) => return Err(format!("Row {} parse error: {:?}", row_idx + 1, e)),
            None => return Err(format!("Row {} is empty", row_idx + 1)),
        };

        result.push(parsed);
    }

    Ok(result)
}


fn parse_json(path: &Path) -> Result<Vec<Vec<f64>>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;

    let v: Value = serde_json::from_str(&contents).map_err(|e| e.to_string())?;
    if let Value::Array(arr) = v {
        arr.iter()
            .map(|row| {
                if let Value::Array(nums) = row {
                    nums.iter()
                        .map(|num| num.as_f64().ok_or("Invalid number".to_string()))
                        .collect()
                } else {
                    Err("Invalid matrix format".to_string())
                }
            })
            .collect()
    } else {
        Err("Invalid JSON format".to_string())
    }
}

fn parse_xml(path: &Path) -> Result<Vec<Vec<f64>>, String> {
    let mut reader = XmlReader::from_file(path).map_err(|e| e.to_string())?;
    reader.trim_text(true);

    let mut matrix = Vec::new();
    let mut current_row = Vec::new();
    let mut buf = Vec::new();
    let mut in_row = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"row" {
                    in_row = true;
                    current_row = Vec::new();
                }
            },
            Ok(Event::Text(e)) => {
                if in_row {
                    let text = e.unescape().map_err(|e| e.to_string())?;
                    // Split the text by whitespace and parse each number
                    for num_str in text.split_whitespace() {
                        let num = num_str.parse::<f64>()
                            .map_err(|_| format!("Invalid number: {}", num_str))?;
                        current_row.push(num);
                    }
                }
            },
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"row" {
                    in_row = false;
                    if !current_row.is_empty() {
                        matrix.push(current_row.clone());
                    }
                }
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing XML: {}", e)),
            _ => (),
        }
    }

    if matrix.is_empty() {
        return Err("No valid matrix data found in XML".to_string());
    }

    Ok(matrix)
}

fn parse_txt(path: &Path) -> Result<Vec<Vec<f64>>, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| e.to_string())?;

    contents
        .lines()
        .map(|line| {
            line.split_whitespace()
                .map(|num| num.parse::<f64>().map_err(|e| e.to_string()))
                .collect()
        })
        .collect()
}