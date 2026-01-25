// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Event bus for inter-component communication

use std::sync::Arc;
use tokio::sync::{broadcast, mpsc};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use crate::sensors::SensorReading;
use crate::detection::Detection;

/// Event types in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    SensorReading,
    Detection,
    Alert,
    SystemStatus,
    Error,
}

/// Generic event wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: u64,
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub payload: EventPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventPayload {
    Reading(SensorReading),
    Detection(Detection),
    Alert { level: String, message: String },
    Status { key: String, value: String },
    Error { code: u32, message: String },
}

/// Central event bus for pub/sub communication
pub struct EventBus {
    reading_tx: broadcast::Sender<SensorReading>,
    detection_tx: broadcast::Sender<Detection>,
    event_tx: broadcast::Sender<Event>,
    event_counter: std::sync::atomic::AtomicU64,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (reading_tx, _) = broadcast::channel(capacity);
        let (detection_tx, _) = broadcast::channel(capacity);
        let (event_tx, _) = broadcast::channel(capacity);
        
        Self {
            reading_tx,
            detection_tx,
            event_tx,
            event_counter: std::sync::atomic::AtomicU64::new(0),
        }
    }
    
    pub fn publish_reading(&self, reading: SensorReading) {
        let _ = self.reading_tx.send(reading.clone());
        self.publish_event(EventType::SensorReading, EventPayload::Reading(reading));
    }
    
    pub fn publish_detection(&self, detection: Detection) {
        let _ = self.detection_tx.send(detection.clone());
        self.publish_event(EventType::Detection, EventPayload::Detection(detection));
    }
    
    pub fn publish_alert(&self, level: &str, message: &str) {
        self.publish_event(
            EventType::Alert,
            EventPayload::Alert {
                level: level.to_string(),
                message: message.to_string(),
            },
        );
    }
    
    pub fn publish_error(&self, code: u32, message: &str) {
        self.publish_event(
            EventType::Error,
            EventPayload::Error {
                code,
                message: message.to_string(),
            },
        );
    }
    
    fn publish_event(&self, event_type: EventType, payload: EventPayload) {
        let id = self.event_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let event = Event {
            id,
            event_type,
            timestamp: Utc::now(),
            payload,
        };
        let _ = self.event_tx.send(event);
    }
    
    pub fn subscribe_readings(&self) -> broadcast::Receiver<SensorReading> {
        self.reading_tx.subscribe()
    }
    
    pub fn subscribe_detections(&self) -> broadcast::Receiver<Detection> {
        self.detection_tx.subscribe()
    }
    
    pub fn subscribe_events(&self) -> broadcast::Receiver<Event> {
        self.event_tx.subscribe()
    }
}
