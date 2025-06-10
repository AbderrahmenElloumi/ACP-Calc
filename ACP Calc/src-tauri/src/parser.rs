use calamine::{open_workbook, Reader, Xlsx};
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
        Some("xlsx") | Some("xls") => parse_excel(path),
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

fn parse_excel(path: &Path) -> Result<Vec<Vec<f64>>, String> {
    let mut workbook: Xlsx<_> = open_workbook(path).map_err(|e: calamine::XlsxError| e.to_string())?;
    let sheet = workbook
        .worksheet_range_at(0)
        .ok_or("No sheets found")?
        .map_err(|e: calamine::XlsxError| e.to_string())?;

    let matrix: Result<Vec<Vec<f64>>, _> = sheet
        .rows()
        .map(|row| {
            row.iter()
                .map(|cell| cell.get_float().ok_or("Invalid number".to_string()))
                .collect()
        })
        .collect();

    matrix
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

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"row" {
                    current_row = Vec::new();
                }
            },
            Ok(Event::Text(e)) => {
                if let Ok(num) = e.unescape().map_err(|e| e.to_string())?.parse::<f64>() {
                    current_row.push(num);
                }
            },
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"row" && !current_row.is_empty() {
                    matrix.push(current_row.clone());
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

    // Validate that all rows have the same length
    let row_length = matrix[0].len();
    if !matrix.iter().all(|row| row.len() == row_length) {
        return Err("Inconsistent row lengths in XML matrix".to_string());
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