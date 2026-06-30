use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;
use std::collections::HashMap;

use crate::mappings;

#[derive(Debug, Clone, Serialize)]
pub struct RespEvent {
    pub resp_event_id: u32,
    pub name: String,
    pub end_time: u32,
    pub duration: u32,
    pub pressure: u32,
    pub strength: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeviceEvent {
    pub device_event_id: u32,
    pub parameter_id: u32,
    pub parameter_name: String,
    pub new_value: String,
    pub time: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub key: String,
    pub resp_events: Vec<RespEvent>,
    pub device_events: Vec<DeviceEvent>,
    pub config: HashMap<String, String>,
    pub therapy_seconds: u32,
    pub file_duration: u32,
    pub has_therapy_data: bool,
}

impl Session {
    pub fn therapy_hours(&self) -> f64 {
        self.therapy_seconds as f64 / 3600.0
    }

    pub fn apnea_count(&self) -> u32 {
        let ids = mappings::ahi_apnea_ids();
        self.resp_events
            .iter()
            .filter(|e| ids.contains(&e.resp_event_id))
            .count() as u32
    }

    pub fn hypopnea_count(&self) -> u32 {
        let ids = mappings::ahi_hypopnea_ids();
        self.resp_events
            .iter()
            .filter(|e| ids.contains(&e.resp_event_id))
            .count() as u32
    }

    pub fn rera_count(&self) -> u32 {
        self.resp_events
            .iter()
            .filter(|e| e.resp_event_id == 121)
            .count() as u32
    }

    pub fn snore_count(&self) -> u32 {
        self.resp_events
            .iter()
            .filter(|e| e.resp_event_id == 131)
            .count() as u32
    }

    pub fn ahi(&self) -> f64 {
        let hours = self.therapy_hours();
        if hours <= 0.0 {
            return 0.0;
        }
        (self.apnea_count() + self.hypopnea_count()) as f64 / hours
    }

    pub fn clinical_events(&self) -> Vec<&RespEvent> {
        self.resp_events
            .iter()
            .filter(|e| mappings::is_clinical_event(e.resp_event_id))
            .collect()
    }

    pub fn alert_count(&self) -> u32 {
        self.resp_events
            .iter()
            .filter(|e| (301..=322).contains(&e.resp_event_id))
            .count() as u32
    }

    pub fn disconnection_count(&self) -> u32 {
        self.resp_events
            .iter()
            .filter(|e| e.resp_event_id == 171)
            .count() as u32
    }

    pub fn desaturation_count(&self) -> u32 {
        self.resp_events
            .iter()
            .filter(|e| e.resp_event_id == 251 || e.resp_event_id == 252)
            .count() as u32
    }

    pub fn apnea_subtypes(&self) -> ApneaSubtypes {
        let mut s = ApneaSubtypes::default();
        for e in &self.resp_events {
            match e.resp_event_id {
                101 => s.obstructive += 1,
                102 => s.central += 1,
                103 => s.leakage += 1,
                104 => s.softstart += 1,
                105 => s.high_pressure += 1,
                106 => s.movement += 1,
                _ => {}
            }
        }
        s
    }

    pub fn hypopnea_subtypes(&self) -> HypopneaSubtypes {
        let mut s = HypopneaSubtypes::default();
        for e in &self.resp_events {
            match e.resp_event_id {
                111 => s.obstructive += 1,
                112 => s.central += 1,
                113 => s.leakage += 1,
                114 => s.softstart += 1,
                115 => s.high_pressure += 1,
                116 => s.pos_change += 1,
                _ => {}
            }
        }
        s
    }

    pub fn avg_event_duration(&self) -> f64 {
        let clinical: Vec<&RespEvent> = self
            .resp_events
            .iter()
            .filter(|e| {
                mappings::ahi_apnea_ids().contains(&e.resp_event_id)
                    || mappings::ahi_hypopnea_ids().contains(&e.resp_event_id)
                    || e.resp_event_id == 121
            })
            .collect();
        if clinical.is_empty() {
            return 0.0;
        }
        let total: u32 = clinical.iter().map(|e| e.duration).sum();
        total as f64 / clinical.len() as f64
    }

    pub fn hourly_ahi(&self) -> Vec<f64> {
        let total_hours = (self.therapy_seconds as f64 / 3600.0).ceil() as usize;
        if total_hours == 0 {
            return vec![];
        }
        let apnea_ids = mappings::ahi_apnea_ids();
        let hypopnea_ids = mappings::ahi_hypopnea_ids();
        let base_time = self.resp_events.iter().map(|e| e.end_time).min().unwrap_or(0);
        let mut hourly = vec![0u32; total_hours];
        for e in &self.resp_events {
            if apnea_ids.contains(&e.resp_event_id) || hypopnea_ids.contains(&e.resp_event_id) {
                let relative = e.end_time.saturating_sub(base_time);
                let hour = (relative / 3600) as usize;
                if hour < total_hours {
                    hourly[hour] += 1;
                }
            }
        }
        hourly.into_iter().map(|c| c as f64).collect()
    }

    pub fn event_pressure_data(&self) -> Vec<(f64, f64, &str)> {
        let apnea_ids = mappings::ahi_apnea_ids();
        let hypopnea_ids = mappings::ahi_hypopnea_ids();
        self.resp_events
            .iter()
            .filter(|e| {
                apnea_ids.contains(&e.resp_event_id)
                    || hypopnea_ids.contains(&e.resp_event_id)
                    || e.resp_event_id == 121
            })
            .filter(|e| e.pressure > 0)
            .map(|e| {
                let cat = if apnea_ids.contains(&e.resp_event_id) {
                    "Apnea"
                } else if hypopnea_ids.contains(&e.resp_event_id) {
                    "Hypopnea"
                } else {
                    "RERA"
                };
                (e.pressure as f64 / 100.0, e.duration as f64, cat)
            })
            .collect()
    }

    pub fn duration_histogram(&self) -> HashMap<String, Vec<u32>> {
        let apnea_ids = mappings::ahi_apnea_ids();
        let hypopnea_ids = mappings::ahi_hypopnea_ids();
        let mut result: HashMap<String, Vec<u32>> = HashMap::new();
        for e in &self.resp_events {
            let cat = if apnea_ids.contains(&e.resp_event_id) {
                "Apnea"
            } else if hypopnea_ids.contains(&e.resp_event_id) {
                "Hypopnea"
            } else if e.resp_event_id == 121 {
                "RERA"
            } else {
                continue;
            };
            result
                .entry(cat.to_string())
                .or_default()
                .push(e.duration);
        }
        result
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ApneaSubtypes {
    pub obstructive: u32,
    pub central: u32,
    pub leakage: u32,
    pub softstart: u32,
    pub high_pressure: u32,
    pub movement: u32,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct HypopneaSubtypes {
    pub obstructive: u32,
    pub central: u32,
    pub leakage: u32,
    pub softstart: u32,
    pub high_pressure: u32,
    pub pos_change: u32,
}

const GAP_THRESHOLD: u32 = 900; // 15 minutes
const MIN_SESSION_SECS: u32 = 600; // 10 minutes
const MIN_NIGHT_SECS: u32 = 7200; // 2 hours minimum for a night

fn is_therapy_event(id: u32) -> bool {
    (100..=999).contains(&id)
}


fn parse_event_xml_raw(key: &str, xml: &str) -> Result<(Vec<RespEvent>, Vec<DeviceEvent>, HashMap<String, String>)> {
    let mut reader = Reader::from_str(xml);
    let mut resp_events = Vec::new();
    let mut device_events = Vec::new();
    let mut config = HashMap::new();

    loop {
        match reader.read_event() {
            Ok(Event::Empty(ref e)) => {
                let tag = e.name();
                let tag_str = std::str::from_utf8(tag.as_ref()).unwrap_or("");

                if tag_str == "RespEvent" {
                    resp_events.push(parse_resp_event(e)?);
                } else if tag_str == "DeviceEvent" {
                    let de = parse_device_event(e)?;
                    if de.device_event_id == 0 {
                        config.insert(de.parameter_name.clone(), de.new_value.clone());
                    }
                    device_events.push(de);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow::anyhow!("XML parse error in file {}: {}", key, e)),
            _ => {}
        }
    }

    Ok((resp_events, device_events, config))
}

fn split_into_sessions(
    file_key: &str,
    resp_events: Vec<RespEvent>,
    device_events: Vec<DeviceEvent>,
    config: HashMap<String, String>,
) -> Vec<Session> {
    let mut therapy_times: Vec<u32> = resp_events
        .iter()
        .filter(|e| is_therapy_event(e.resp_event_id))
        .map(|e| e.end_time)
        .collect();
    therapy_times.sort();
    therapy_times.dedup();

    // File-level recording duration = max EndTime across all events
    let file_duration = resp_events.iter().map(|e| e.end_time).max().unwrap_or(0);

    if therapy_times.is_empty() {
        return vec![];
    }

    let mut boundaries: Vec<(u32, u32)> = Vec::new();
    let mut seg_start = therapy_times[0];
    let mut prev = therapy_times[0];

    for &t in &therapy_times[1..] {
        if t - prev > GAP_THRESHOLD {
            boundaries.push((seg_start, prev));
            seg_start = t;
        }
        prev = t;
    }
    boundaries.push((seg_start, prev));

    let boundaries: Vec<(u32, u32)> = boundaries
        .into_iter()
        .filter(|(s, e)| e - s >= MIN_SESSION_SECS)
        .collect();

    if boundaries.is_empty() {
        return vec![];
    }

    if boundaries.len() == 1 {
        let (start, end) = boundaries[0];
        return vec![Session {
            key: file_key.to_string(),
            resp_events,
            device_events,
            config,
            therapy_seconds: end - start,
            file_duration,
            has_therapy_data: true,
        }];
    }

    boundaries
        .iter()
        .enumerate()
        .map(|(i, &(seg_start, seg_end))| {
            let sub_key = format!("{}.{}", file_key, i + 1);
            let sub_resp: Vec<RespEvent> = resp_events
                .iter()
                .filter(|e| e.end_time >= seg_start && e.end_time <= seg_end + GAP_THRESHOLD)
                .cloned()
                .collect();
            let sub_dev: Vec<DeviceEvent> = device_events
                .iter()
                .filter(|e| e.time >= seg_start && e.time <= seg_end + GAP_THRESHOLD)
                .cloned()
                .collect();
            Session {
                key: sub_key,
                resp_events: sub_resp,
                device_events: sub_dev,
                config: config.clone(),
                therapy_seconds: seg_end - seg_start,
                file_duration,
                has_therapy_data: true,
            }
        })
        .collect()
}

fn parse_resp_event(e: &quick_xml::events::BytesStart) -> Result<RespEvent> {
    let mut resp_event_id = 0u32;
    let mut end_time = 0u32;
    let mut duration = 0u32;
    let mut pressure = 0u32;
    let mut strength = 0u32;

    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
        let val = std::str::from_utf8(&attr.value).unwrap_or("0");
        match key {
            "RespEventID" => resp_event_id = val.parse().unwrap_or(0),
            "EndTime" => end_time = val.parse().unwrap_or(0),
            "Duration" => duration = val.parse().unwrap_or(0),
            "Pressure" => pressure = val.parse().unwrap_or(0),
            "Strength" => strength = val.parse().unwrap_or(0),
            _ => {}
        }
    }

    // EndTime and Duration are stored in deciseconds (1/10s); convert to seconds
    Ok(RespEvent {
        resp_event_id,
        name: mappings::resp_event_name(resp_event_id).to_string(),
        end_time: end_time / 10,
        duration: duration / 10,
        pressure,
        strength,
    })
}

fn parse_device_event(e: &quick_xml::events::BytesStart) -> Result<DeviceEvent> {
    let mut device_event_id = 0u32;
    let mut parameter_id = 0u32;
    let mut new_value = String::new();
    let mut time = 0u32;

    for attr in e.attributes().flatten() {
        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
        let val = std::str::from_utf8(&attr.value).unwrap_or("");
        match key {
            "DeviceEventID" => device_event_id = val.parse().unwrap_or(0),
            "ParameterID" => parameter_id = val.parse().unwrap_or(0),
            "NewValue" => new_value = val.to_string(),
            "Time" => time = val.parse().unwrap_or(0),
            _ => {}
        }
    }

    Ok(DeviceEvent {
        device_event_id,
        parameter_id,
        parameter_name: mappings::param_name(parameter_id).to_string(),
        new_value,
        time,
    })
}

pub fn parse_all_events(event_files: &HashMap<String, String>) -> Result<Vec<Session>> {
    let mut sessions: Vec<Session> = Vec::new();

    for (key, xml) in event_files {
        let (resp, dev, config) = parse_event_xml_raw(key, xml)
            .with_context(|| format!("Failed to parse file {}", key))?;
        let mut subs = split_into_sessions(key, resp, dev, config);
        sessions.append(&mut subs);
    }

    sessions.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(sessions)
}

#[derive(Debug, Clone, Serialize)]
pub struct Night {
    pub key: String,
    pub session_keys: Vec<String>,
    pub therapy_seconds: u32,
    pub resp_events: Vec<RespEvent>,
    pub device_events: Vec<DeviceEvent>,
    pub config: HashMap<String, String>,
    pub has_therapy_data: bool,
}

impl Night {
    pub fn therapy_hours(&self) -> f64 {
        self.therapy_seconds as f64 / 3600.0
    }

    pub fn apnea_count(&self) -> u32 {
        let ids = mappings::ahi_apnea_ids();
        self.resp_events.iter().filter(|e| ids.contains(&e.resp_event_id)).count() as u32
    }

    pub fn hypopnea_count(&self) -> u32 {
        let ids = mappings::ahi_hypopnea_ids();
        self.resp_events.iter().filter(|e| ids.contains(&e.resp_event_id)).count() as u32
    }

    pub fn rera_count(&self) -> u32 {
        self.resp_events.iter().filter(|e| e.resp_event_id == 121).count() as u32
    }

    pub fn snore_count(&self) -> u32 {
        self.resp_events.iter().filter(|e| e.resp_event_id == 131).count() as u32
    }

    pub fn alert_count(&self) -> u32 {
        self.resp_events.iter().filter(|e| (301..=322).contains(&e.resp_event_id)).count() as u32
    }

    pub fn ahi(&self) -> f64 {
        let hours = self.therapy_hours();
        if hours <= 0.0 { return 0.0; }
        (self.apnea_count() + self.hypopnea_count()) as f64 / hours
    }
}


fn date_from_key(key: &str) -> String {
    if key.len() >= 8 && key.starts_with("20") {
        key[..8].to_string()
    } else {
        "unknown".to_string()
    }
}

pub fn group_into_nights(sessions: &[Session]) -> Vec<Night> {
    let mut by_date: HashMap<String, Vec<&Session>> = HashMap::new();
    for s in sessions {
        let date = date_from_key(&s.key);
        by_date.entry(date).or_default().push(s);
    }

    let mut nights = Vec::new();

    for (date, date_sessions) in &by_date {
        use std::collections::HashSet;
        let mut seen: HashSet<(u32, u32, u32)> = HashSet::new();
        let mut merged_resp: Vec<RespEvent> = Vec::new();
        let mut merged_dev: Vec<DeviceEvent> = Vec::new();
        let mut config: HashMap<String, String> = HashMap::new();
        let mut session_keys: Vec<String> = Vec::new();

        for s in date_sessions {
            session_keys.push(s.key.clone());
            config.extend(s.config.clone());
            merged_dev.extend(s.device_events.clone());
            for e in &s.resp_events {
                let key = (e.end_time, e.resp_event_id, e.duration);
                if seen.insert(key) {
                    merged_resp.push(e.clone());
                }
            }
        }

        // Therapy duration = sum of each FILE's recording duration.
        // Each event file has its own time base (seconds from session start).
        // Sessions split from the same file share the same file_duration,
        // so dedup by base key.
        let mut file_durations: HashMap<String, u32> = HashMap::new();
        for s in date_sessions.iter() {
            let base_key = if let Some(dot_pos) = s.key.rfind('.') {
                if s.key[dot_pos + 1..].chars().all(|c| c.is_ascii_digit()) {
                    &s.key[..dot_pos]
                } else {
                    &s.key
                }
            } else {
                &s.key
            };
            file_durations
                .entry(base_key.to_string())
                .or_insert(s.file_duration);
        }
        let therapy_secs: u32 = file_durations.values().sum();

        if therapy_secs < MIN_NIGHT_SECS {
            continue;
        }

        nights.push(Night {
            key: date.clone(),
            session_keys,
            therapy_seconds: therapy_secs,
            resp_events: merged_resp,
            device_events: merged_dev,
            config,
            has_therapy_data: true,
        });
    }

    nights.sort_by(|a, b| a.key.cmp(&b.key));
    nights
}
