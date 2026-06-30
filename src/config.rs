use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;

use crate::mappings;

#[derive(Debug, Clone, Serialize, Default)]
pub struct DeviceInfo {
    pub device_type: String,
    pub serial_number: String,
    pub fw_version: String,
    pub fw_build: String,
    pub pm_version: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct TherapyConfig {
    pub mode: String,
    pub epap: f64,
    pub ipap: f64,
    pub ipap_max: f64,
    pub eepap_min: f64,
    pub eepap_max: f64,
    pub humidifier_level: String,
    pub auto_start: bool,
    pub auto_stop: bool,
    pub apap_dynamic: bool,
    pub tube_type: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct OperatingTime {
    pub boot_count: u32,
    pub therapy_count: u32,
    pub time_value: u32,
}

pub fn parse_device_xml(xml: &str) -> Result<DeviceInfo> {
    let mut reader = Reader::from_str(xml);
    let mut info = DeviceInfo::default();

    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) => {
                let name = e.name();
                let tag = std::str::from_utf8(name.as_ref()).unwrap_or("");
                let value = get_value_attr(e);
                match tag {
                    "DeviceType" => info.device_type = value,
                    "DeviceSerialNumber" => info.serial_number = value,
                    "FWVersion" => info.fw_version = value,
                    "FWBuild" => info.fw_build = value,
                    "PMVersion" => info.pm_version = value,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    Ok(info)
}

pub fn parse_therapy_config(session_config: &std::collections::HashMap<String, String>) -> TherapyConfig {
    let mode_val: u32 = session_config
        .get("Mode")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    TherapyConfig {
        mode: mappings::mode_name(mode_val).to_string(),
        epap: parse_pressure(session_config.get("Epap")),
        ipap: parse_pressure(session_config.get("IPap")),
        ipap_max: parse_pressure(session_config.get("IPapMax")),
        eepap_min: parse_pressure(session_config.get("EepapMin")),
        eepap_max: parse_pressure(session_config.get("EepapMax")),
        humidifier_level: session_config
            .get("HumidifierLevel")
            .cloned()
            .unwrap_or_default(),
        auto_start: session_config.get("AutoStart").map(|v| v == "1").unwrap_or(false),
        auto_stop: session_config.get("AutoStop").map(|v| v == "1").unwrap_or(false),
        apap_dynamic: session_config.get("Apap_dyn").map(|v| v == "1").unwrap_or(false),
        tube_type: session_config
            .get("TubeType")
            .cloned()
            .unwrap_or_default(),
    }
}

pub fn parse_operating_time(ini: &str) -> OperatingTime {
    let mut ot = OperatingTime::default();
    for line in ini.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("bootCount=") {
            ot.boot_count = val.parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("therapyCount=") {
            ot.therapy_count = val.parse().unwrap_or(0);
        } else if let Some(val) = line.strip_prefix("timeValue=") {
            ot.time_value = val.parse().unwrap_or(0);
        }
    }
    ot
}

fn get_value_attr(e: &quick_xml::events::BytesStart) -> String {
    for attr in e.attributes().flatten() {
        if std::str::from_utf8(attr.key.as_ref()) == Ok("value") {
            return std::str::from_utf8(&attr.value).unwrap_or("").to_string();
        }
    }
    String::new()
}

fn parse_pressure(val: Option<&String>) -> f64 {
    val.and_then(|v| v.parse::<f64>().ok())
        .map(|v| v / 100.0)
        .unwrap_or(0.0)
}
