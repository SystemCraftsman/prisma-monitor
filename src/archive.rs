use anyhow::{Context, Result};
use std::collections::HashMap;
use std::io::Read;

pub struct ExtractedData {
    pub event_files: HashMap<String, String>,
    pub device_xml: Option<String>,
    pub config_xml: Option<String>,
    pub operating_time: Option<String>,
}

pub fn extract_pdat(data: &[u8]) -> Result<ExtractedData> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).context("Invalid .pdat file (not a valid ZIP)")?;

    let mut event_files = HashMap::new();
    let mut device_xml = None;
    let config_xml = None;
    let mut operating_time = None;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name.contains("event_") && name.ends_with(".xml") {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            let key = extract_session_key(&name);
            event_files.insert(key, content);
        } else if name.ends_with("device.xml") {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            device_xml = Some(content);
        } else if name.ends_with("config_operatingtime.ini") {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            operating_time = Some(content);
        }
    }

    Ok(ExtractedData {
        event_files,
        device_xml,
        config_xml,
        operating_time,
    })
}

pub fn extract_pcfg(data: &[u8]) -> Result<Option<String>> {
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).context("Invalid .pcfg file (not a valid ZIP)")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        if name.ends_with("configuration.xml") {
            let mut content = String::new();
            file.read_to_string(&mut content)?;
            return Ok(Some(content));
        }
    }

    Ok(None)
}

fn extract_session_key(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let filename = parts.last().unwrap_or(&path);
    let file_num = filename
        .trim_start_matches("event_")
        .trim_end_matches(".xml");

    // Look for a date directory (8-digit, starts with 20) in the path
    for part in &parts {
        if part.len() == 8 && part.starts_with("20") && part.chars().all(|c| c.is_ascii_digit()) {
            return format!("{}_{}", part, file_num);
        }
    }

    file_num.to_string()
}
