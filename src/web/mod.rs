pub mod routes;
pub mod templates;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::config::{DeviceInfo, OperatingTime, TherapyConfig};
use crate::events::{Session, Night};

#[derive(Default)]
pub struct AppState {
    pub sessions: Vec<Session>,
    pub nights: Vec<Night>,
    pub device_info: Option<DeviceInfo>,
    pub therapy_config: Option<TherapyConfig>,
    pub operating_time: Option<OperatingTime>,
    pub config_xml: Option<String>,
}

pub type SharedState = Arc<RwLock<AppState>>;

pub fn new_shared_state() -> SharedState {
    Arc::new(RwLock::new(AppState::default()))
}
