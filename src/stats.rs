use serde::Serialize;

use crate::events::{Session, Night};
use crate::mappings;

#[derive(Debug, Clone, Serialize)]
pub struct OverallStats {
    pub total_sessions: usize,
    pub sessions_with_data: usize,
    pub total_therapy_hours: f64,
    pub avg_session_hours: f64,
    pub total_apneas: u32,
    pub total_hypopneas: u32,
    pub total_reras: u32,
    pub total_snoring: u32,
    pub total_alerts: u32,
    pub total_desaturations: u32,
    pub ahi: f64,
    pub rdi: f64,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionSummary {
    pub key: String,
    pub display_date: String,
    pub therapy_hours: f64,
    pub apneas: u32,
    pub hypopneas: u32,
    pub reras: u32,
    pub snoring: u32,
    pub alerts: u32,
    pub ahi: f64,
    pub severity: String,
    pub has_data: bool,
}

fn format_session_date(key: &str) -> String {
    if key.len() >= 8 && key.starts_with("20") {
        let date_part = &key[..8];
        if let (Ok(y), Ok(m), Ok(d)) = (
            date_part[..4].parse::<u32>(),
            date_part[4..6].parse::<u32>(),
            date_part[6..8].parse::<u32>(),
        ) {
            let month = match m {
                1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
                5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
                9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
                _ => "???",
            };
            return format!("{} {:02}, {}", month, d, y);
        }
    }
    String::new()
}

pub fn compute_overall(sessions: &[Session]) -> OverallStats {
    let sessions_with_data: Vec<&Session> = sessions
        .iter()
        .filter(|s| s.has_therapy_data && s.therapy_hours() >= 0.5)
        .collect();

    let total_therapy_hours: f64 = sessions_with_data.iter().map(|s| s.therapy_hours()).sum();
    let avg_session_hours = if !sessions_with_data.is_empty() {
        total_therapy_hours / sessions_with_data.len() as f64
    } else {
        0.0
    };
    let total_apneas: u32 = sessions_with_data.iter().map(|s| s.apnea_count()).sum();
    let total_hypopneas: u32 = sessions_with_data.iter().map(|s| s.hypopnea_count()).sum();
    let total_reras: u32 = sessions_with_data.iter().map(|s| s.rera_count()).sum();
    let total_snoring: u32 = sessions_with_data.iter().map(|s| s.snore_count()).sum();
    let total_alerts: u32 = sessions_with_data.iter().map(|s| s.alert_count()).sum();
    let total_desaturations: u32 = sessions_with_data.iter().map(|s| s.desaturation_count()).sum();

    let ahi = if total_therapy_hours > 0.0 {
        (total_apneas + total_hypopneas) as f64 / total_therapy_hours
    } else {
        0.0
    };

    let rdi = if total_therapy_hours > 0.0 {
        (total_apneas + total_hypopneas + total_reras) as f64 / total_therapy_hours
    } else {
        0.0
    };

    OverallStats {
        total_sessions: sessions.len(),
        sessions_with_data: sessions_with_data.len(),
        total_therapy_hours,
        avg_session_hours,
        total_apneas,
        total_hypopneas,
        total_reras,
        total_snoring,
        total_alerts,
        total_desaturations,
        ahi,
        rdi,
        severity: mappings::ahi_severity(ahi).to_string(),
    }
}

pub fn session_summaries(sessions: &[Session]) -> Vec<SessionSummary> {
    sessions
        .iter()
        .map(|s| SessionSummary {
            display_date: format_session_date(&s.key),
            key: s.key.clone(),
            therapy_hours: s.therapy_hours(),
            apneas: s.apnea_count(),
            hypopneas: s.hypopnea_count(),
            reras: s.rera_count(),
            snoring: s.snore_count(),
            alerts: s.alert_count(),
            ahi: s.ahi(),
            severity: mappings::ahi_severity(s.ahi()).to_string(),
            has_data: s.has_therapy_data,
        })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct NightSummary {
    pub key: String,
    pub display_date: String,
    pub session_count: usize,
    pub therapy_hours: f64,
    pub apneas: u32,
    pub hypopneas: u32,
    pub reras: u32,
    pub snoring: u32,
    pub alerts: u32,
    pub ahi: f64,
    pub severity: String,
    pub has_data: bool,
}

pub fn night_summaries(nights: &[Night]) -> Vec<NightSummary> {
    nights
        .iter()
        .map(|n| {
            let ahi = n.ahi();
            NightSummary {
                display_date: format_session_date(&n.key),
                key: n.key.clone(),
                session_count: n.session_keys.len(),
                therapy_hours: n.therapy_hours(),
                apneas: n.apnea_count(),
                hypopneas: n.hypopnea_count(),
                reras: n.rera_count(),
                snoring: n.snore_count(),
                alerts: n.alert_count(),
                ahi,
                severity: mappings::ahi_severity(ahi).to_string(),
                has_data: n.has_therapy_data,
            }
        })
        .collect()
}

pub fn compute_overall_from_nights(nights: &[Night]) -> OverallStats {
    let data_nights: Vec<&Night> = nights
        .iter()
        .filter(|n| n.has_therapy_data && n.therapy_hours() >= 0.5)
        .collect();

    let total_therapy_hours: f64 = data_nights.iter().map(|n| n.therapy_hours()).sum();
    let avg_session_hours = if !data_nights.is_empty() {
        total_therapy_hours / data_nights.len() as f64
    } else {
        0.0
    };
    let total_apneas: u32 = data_nights.iter().map(|n| n.apnea_count()).sum();
    let total_hypopneas: u32 = data_nights.iter().map(|n| n.hypopnea_count()).sum();
    let total_reras: u32 = data_nights.iter().map(|n| n.rera_count()).sum();
    let total_snoring: u32 = data_nights.iter().map(|n| n.snore_count()).sum();
    let total_alerts: u32 = data_nights.iter().map(|n| n.alert_count()).sum();
    let total_desaturations: u32 = 0;

    let ahi = if total_therapy_hours > 0.0 {
        (total_apneas + total_hypopneas) as f64 / total_therapy_hours
    } else {
        0.0
    };

    let rdi = if total_therapy_hours > 0.0 {
        (total_apneas + total_hypopneas + total_reras) as f64 / total_therapy_hours
    } else {
        0.0
    };

    OverallStats {
        total_sessions: nights.len(),
        sessions_with_data: data_nights.len(),
        total_therapy_hours,
        avg_session_hours,
        total_apneas,
        total_hypopneas,
        total_reras,
        total_snoring,
        total_alerts,
        total_desaturations,
        ahi,
        rdi,
        severity: mappings::ahi_severity(ahi).to_string(),
    }
}
