use axum::extract::{Multipart, Path, State};
use axum::response::{Html, IntoResponse, Redirect};
use crate::archive;
use crate::config;
use crate::events;
use crate::mappings;
use crate::stats;
use crate::web::templates::*;
use crate::web::SharedState;

pub async fn upload_page(State(state): State<SharedState>) -> impl IntoResponse {
    let s = state.read().await;
    let tmpl = UploadTemplate {
        has_data: !s.sessions.is_empty(),
    };
    Html(tmpl.to_string())
}

pub async fn handle_upload(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().unwrap_or("").to_string();
        let data = match field.bytes().await {
            Ok(d) => d,
            Err(_) => continue,
        };

        if name == "pdat" || file_name.ends_with(".pdat") {
            match archive::extract_pdat(&data) {
                Ok(extracted) => {
                    let sessions = match events::parse_all_events(&extracted.event_files) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    let nights = events::group_into_nights(&sessions);

                    let device_info = extracted
                        .device_xml
                        .as_deref()
                        .and_then(|xml| config::parse_device_xml(xml).ok());

                    let therapy_config = sessions
                        .iter()
                        .filter(|s| s.has_therapy_data)
                        .last()
                        .map(|s| config::parse_therapy_config(&s.config));

                    let operating_time = extracted
                        .operating_time
                        .as_deref()
                        .map(config::parse_operating_time);

                    let mut s = state.write().await;
                    s.sessions = sessions;
                    s.nights = nights;
                    s.device_info = device_info;
                    s.therapy_config = therapy_config;
                    s.operating_time = operating_time;
                }
                Err(_) => continue,
            }
        } else if name == "pcfg" || file_name.ends_with(".pcfg") {
            if let Ok(Some(xml)) = archive::extract_pcfg(&data) {
                let mut s = state.write().await;
                s.config_xml = Some(xml);
            }
        }
    }

    Redirect::to("/")
}

pub async fn dashboard(State(state): State<SharedState>) -> impl IntoResponse {
    let s = state.read().await;

    if s.sessions.is_empty() {
        return Redirect::to("/upload").into_response();
    }

    let overall = stats::compute_overall_from_nights(&s.nights);
    let summaries = stats::night_summaries(&s.nights);

    let data_nights: Vec<&crate::stats::NightSummary> =
        summaries.iter().filter(|n| n.has_data && n.therapy_hours >= 0.5).collect();

    let labels: Vec<String> = data_nights.iter().map(|n| {
        if n.display_date.is_empty() { format!("#{}", n.key) } else { n.display_date.clone() }
    }).collect();
    let ahis: Vec<f64> = data_nights.iter().map(|n| round2(n.ahi)).collect();
    let rdis: Vec<f64> = data_nights
        .iter()
        .map(|n| {
            if n.therapy_hours > 0.0 {
                round2((n.apneas + n.hypopneas + n.reras) as f64 / n.therapy_hours)
            } else {
                0.0
            }
        })
        .collect();
    let hours: Vec<f64> = data_nights
        .iter()
        .map(|n| round2(n.therapy_hours))
        .collect();
    let apneas: Vec<u32> = data_nights.iter().map(|n| n.apneas).collect();
    let hypopneas: Vec<u32> = data_nights.iter().map(|n| n.hypopneas).collect();
    let reras: Vec<u32> = data_nights.iter().map(|n| n.reras).collect();
    let snoring: Vec<u32> = data_nights.iter().map(|n| n.snoring).collect();

    let recent: Vec<crate::stats::NightSummary> = data_nights
        .iter()
        .rev()
        .take(5)
        .rev()
        .cloned()
        .cloned()
        .collect();

    let device_name = s
        .device_info
        .as_ref()
        .map(|d| format!("Prisma (SN: {})", d.serial_number))
        .unwrap_or_else(|| "Unknown device".to_string());

    let tmpl = DashboardTemplate {
        stats: overall,
        session_labels_json: serde_json::to_string(&labels).unwrap_or_default(),
        session_ahi_json: serde_json::to_string(&ahis).unwrap_or_default(),
        session_rdi_json: serde_json::to_string(&rdis).unwrap_or_default(),
        session_hours_json: serde_json::to_string(&hours).unwrap_or_default(),
        session_apneas_json: serde_json::to_string(&apneas).unwrap_or_default(),
        session_hypopneas_json: serde_json::to_string(&hypopneas).unwrap_or_default(),
        session_reras_json: serde_json::to_string(&reras).unwrap_or_default(),
        session_snoring_json: serde_json::to_string(&snoring).unwrap_or_default(),
        device_name,
        recent_sessions: recent,
    };

    Html(tmpl.to_string()).into_response()
}

pub async fn sessions_page(State(state): State<SharedState>) -> impl IntoResponse {
    let s = state.read().await;

    if s.sessions.is_empty() {
        return Redirect::to("/upload").into_response();
    }

    let mut summaries = stats::night_summaries(&s.nights);
    summaries.reverse();
    let data_only: Vec<&crate::stats::NightSummary> =
        summaries.iter().filter(|n| n.has_data).collect();
    let max_apneas = data_only.iter().map(|n| n.apneas).max().unwrap_or(1).max(1);
    let max_hypopneas = data_only
        .iter()
        .map(|n| n.hypopneas)
        .max()
        .unwrap_or(1)
        .max(1);
    let max_snoring = data_only
        .iter()
        .map(|n| n.snoring)
        .max()
        .unwrap_or(1)
        .max(1);

    let tmpl = SessionsTemplate {
        summaries,
        max_apneas,
        max_hypopneas,
        max_snoring,
    };
    Html(tmpl.to_string()).into_response()
}

pub async fn session_detail(
    State(state): State<SharedState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    let s = state.read().await;

    let session = match s.sessions.iter().find(|s| s.key == key) {
        Some(s) => s,
        None => return Redirect::to("/sessions").into_response(),
    };

    let apnea_sub = session.apnea_subtypes();
    let hypopnea_sub = session.hypopnea_subtypes();

    let apnea_subtypes = serde_json::json!({
        "labels": ["Obstructive", "Central", "Leakage", "Softstart", "High Press.", "Movement"],
        "values": [apnea_sub.obstructive, apnea_sub.central, apnea_sub.leakage,
                   apnea_sub.softstart, apnea_sub.high_pressure, apnea_sub.movement]
    });
    let hypopnea_subtypes = serde_json::json!({
        "labels": ["Obstructive", "Central", "Leakage", "Softstart", "High Press.", "Pos. Change"],
        "values": [hypopnea_sub.obstructive, hypopnea_sub.central, hypopnea_sub.leakage,
                   hypopnea_sub.softstart, hypopnea_sub.high_pressure, hypopnea_sub.pos_change]
    });

    let clinical = session.clinical_events();
    let apnea_ids = mappings::ahi_apnea_ids();
    let hypopnea_ids = mappings::ahi_hypopnea_ids();

    let mut timeline: Vec<serde_json::Value> = Vec::new();
    for e in &clinical {
        let category = if apnea_ids.contains(&e.resp_event_id) {
            "Apnea"
        } else if hypopnea_ids.contains(&e.resp_event_id) {
            "Hypopnea"
        } else if e.resp_event_id == 121 {
            "RERA"
        } else if e.resp_event_id == 131 {
            "Snore"
        } else {
            continue;
        };
        timeline.push(serde_json::json!({
            "time": round2(e.end_time as f64 / 3600.0),
            "category": category,
            "duration": e.duration,
            "name": e.name,
            "pressure": round2(e.pressure as f64 / 100.0),
        }));
    }

    let hourly = session.hourly_ahi();
    let hourly_labels: Vec<String> = (0..hourly.len()).map(|i| format!("Hour {}", i + 1)).collect();

    let pressure_data = session.event_pressure_data();
    let pressure_scatter: Vec<serde_json::Value> = pressure_data
        .iter()
        .map(|(p, d, cat)| {
            serde_json::json!({"x": round2(*p), "y": *d, "category": cat})
        })
        .collect();

    let dur_hist = session.duration_histogram();
    let mut duration_histogram = serde_json::Map::new();
    for (cat, durations) in &dur_hist {
        let bins = compute_duration_bins(durations);
        duration_histogram.insert(cat.clone(), serde_json::to_value(bins).unwrap_or_default());
    }

    let tmpl = SessionDetailTemplate {
        key: session.key.clone(),
        therapy_hours: session.therapy_hours(),
        apneas: session.apnea_count(),
        hypopneas: session.hypopnea_count(),
        reras: session.rera_count(),
        snoring: session.snore_count(),
        alerts: session.alert_count(),
        disconnections: session.disconnection_count(),
        desaturations: session.desaturation_count(),
        avg_duration: round2(session.avg_event_duration()),
        ahi: round2(session.ahi()),
        severity: mappings::ahi_severity(session.ahi()).to_string(),
        apnea_subtypes_json: serde_json::to_string(&apnea_subtypes).unwrap_or_default(),
        hypopnea_subtypes_json: serde_json::to_string(&hypopnea_subtypes).unwrap_or_default(),
        timeline_json: serde_json::to_string(&timeline).unwrap_or_default(),
        hourly_ahi_json: serde_json::to_string(&hourly).unwrap_or_default(),
        hourly_labels_json: serde_json::to_string(&hourly_labels).unwrap_or_default(),
        pressure_scatter_json: serde_json::to_string(&pressure_scatter).unwrap_or_default(),
        duration_histogram_json: serde_json::to_string(&duration_histogram).unwrap_or_default(),
    };

    Html(tmpl.to_string()).into_response()
}

pub async fn config_page(State(state): State<SharedState>) -> impl IntoResponse {
    let s = state.read().await;

    if s.sessions.is_empty() {
        return Redirect::to("/upload").into_response();
    }

    let device = s.device_info.clone().unwrap_or_default();
    let therapy = s.therapy_config.clone().unwrap_or_default();
    let ot = s.operating_time.clone().unwrap_or_default();

    let data_sessions: Vec<&crate::events::Session> = s
        .sessions
        .iter()
        .filter(|s| s.has_therapy_data)
        .collect();

    let mut config_changes: Vec<serde_json::Value> = Vec::new();
    let important_params = [
        "Mode", "Epap", "IPap", "IPapMax", "EepapMin", "EepapMax",
        "HumidifierLevel", "AutoStart", "AutoStop", "Apap_dyn", "TubeType",
    ];

    for i in 1..data_sessions.len() {
        let prev = &data_sessions[i - 1].config;
        let curr = &data_sessions[i].config;
        for param in &important_params {
            let p = param.to_string();
            let prev_val = prev.get(&p).cloned().unwrap_or_default();
            let curr_val = curr.get(&p).cloned().unwrap_or_default();
            if prev_val != curr_val && !curr_val.is_empty() {
                config_changes.push(serde_json::json!({
                    "session": data_sessions[i].key,
                    "param": param,
                    "from": format_config_val(param, &prev_val),
                    "to": format_config_val(param, &curr_val),
                }));
            }
        }
    }

    let tmpl = ConfigTemplate {
        device,
        therapy,
        boot_count: ot.boot_count,
        therapy_count: ot.therapy_count,
        config_changes_json: serde_json::to_string(&config_changes).unwrap_or_default(),
    };

    Html(tmpl.to_string()).into_response()
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn compute_duration_bins(durations: &[u32]) -> Vec<serde_json::Value> {
    let bins = [(0, 10), (10, 20), (20, 30), (30, 45), (45, 60), (60, 300)];
    let labels = ["0-10s", "10-20s", "20-30s", "30-45s", "45-60s", "60s+"];

    bins.iter()
        .zip(labels.iter())
        .map(|((lo, hi), label)| {
            let count = durations.iter().filter(|&&d| d >= *lo && d < *hi).count();
            serde_json::json!({"label": label, "count": count})
        })
        .collect()
}

fn format_config_val(param: &str, val: &str) -> String {
    match param {
        "Mode" => {
            let id: u32 = val.parse().unwrap_or(0);
            mappings::mode_name(id).to_string()
        }
        "Epap" | "IPap" | "IPapMax" | "EepapMin" | "EepapMax" => {
            let v: f64 = val.parse().unwrap_or(0.0);
            format!("{:.1} cmH2O", v / 100.0)
        }
        "AutoStart" | "AutoStop" | "Apap_dyn" => {
            if val == "1" { "Yes".to_string() } else { "No".to_string() }
        }
        _ => val.to_string(),
    }
}
