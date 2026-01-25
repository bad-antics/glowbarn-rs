//! Core engine module - orchestrates all detection systems

mod engine;
mod scheduler;
mod event_bus;

pub use engine::Engine;
pub use scheduler::Scheduler;
pub use event_bus::{EventBus, Event, EventType};

use crate::sensors::SensorReading;
use crate::detection::Detection;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// System-wide state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub running: bool,
    pub sensors_active: usize,
    pub total_readings: u64,
    pub total_detections: u64,
    pub uptime_seconds: u64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub last_detection: Option<DateTime<Utc>>,
}

impl Default for SystemState {
    fn default() -> Self {
        Self {
            running: false,
            sensors_active: 0,
            total_readings: 0,
            total_detections: 0,
            uptime_seconds: 0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            last_detection: None,
        }
    }
}
