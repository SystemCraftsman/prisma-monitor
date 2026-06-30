use askama::Template;

use crate::config::{DeviceInfo, TherapyConfig};
use crate::stats::{NightSummary, OverallStats};

#[derive(Template)]
#[template(path = "upload.html")]
pub struct UploadTemplate {
    pub has_data: bool,
}

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate {
    pub stats: OverallStats,
    pub session_labels_json: String,
    pub session_ahi_json: String,
    pub session_rdi_json: String,
    pub session_hours_json: String,
    pub session_apneas_json: String,
    pub session_hypopneas_json: String,
    pub session_reras_json: String,
    pub session_snoring_json: String,
    pub device_name: String,
    pub recent_sessions: Vec<NightSummary>,
}

#[derive(Template)]
#[template(path = "sessions.html")]
pub struct SessionsTemplate {
    pub summaries: Vec<NightSummary>,
    pub max_apneas: u32,
    pub max_hypopneas: u32,
    pub max_snoring: u32,
}

#[derive(Template)]
#[template(path = "session_detail.html")]
pub struct SessionDetailTemplate {
    pub key: String,
    pub therapy_hours: f64,
    pub apneas: u32,
    pub hypopneas: u32,
    pub reras: u32,
    pub snoring: u32,
    pub alerts: u32,
    pub disconnections: u32,
    pub desaturations: u32,
    pub avg_duration: f64,
    pub ahi: f64,
    pub severity: String,
    pub apnea_subtypes_json: String,
    pub hypopnea_subtypes_json: String,
    pub timeline_json: String,
    pub hourly_ahi_json: String,
    pub hourly_labels_json: String,
    pub pressure_scatter_json: String,
    pub duration_histogram_json: String,
}

#[derive(Template)]
#[template(path = "config.html")]
pub struct ConfigTemplate {
    pub device: DeviceInfo,
    pub therapy: TherapyConfig,
    pub boot_count: u32,
    pub therapy_count: u32,
    pub config_changes_json: String,
}
