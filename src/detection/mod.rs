//! Detection module - sensor fusion and anomaly classification

mod fusion;
mod classification;
mod correlation;

pub use fusion::*;
pub use classification::*;
pub use correlation::*;

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tracing::{info, warn, debug};

use crate::sensors::{SensorReading, SensorType};
use crate::analysis::{EntropyResult, Anomaly, AnomalyType};
use crate::config::Config;
use crate::core::EventBus;

/// Detection event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detection {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub detection_type: DetectionType,
    pub confidence: f64,
    pub severity: Severity,
    
    // Contributing sensors
    pub sensors: Vec<SensorContribution>,
    
    // Analysis data
    pub entropy_deviation: f64,
    pub anomaly_count: usize,
    pub correlation_score: f64,
    
    // Classification
    pub classification: Option<Classification>,
    
    // Location estimate (if available)
    pub location: Option<[f64; 3]>,
    
    // Raw data reference
    pub data_window_start: DateTime<Utc>,
    pub data_window_end: DateTime<Utc>,
}

/// Sensor contribution to detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorContribution {
    pub sensor_id: String,
    pub sensor_type: SensorType,
    pub weight: f64,
    pub reading_value: f64,
    pub anomaly_score: f64,
}

/// Detection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionType {
    // Environmental
    ThermalAnomaly,
    TemperatureGradient,
    ColdSpot,
    HotSpot,
    
    // Acoustic
    InfrasoundEvent,
    UltrasonicEvent,
    EVP,  // Electronic Voice Phenomena
    UnexplainedSound,
    
    // Electromagnetic
    EMFSpike,
    EMFFluctuation,
    MagneticAnomaly,
    StaticDischarge,
    
    // Motion/Vibration
    SeismicEvent,
    Vibration,
    Movement,
    
    // Radiation
    RadiationSpike,
    IonizationChange,
    
    // RF/Electronic
    RFAnomaly,
    InterferencePattern,
    
    // Optical
    LightAnomaly,
    LaserInterruption,
    SpectrumAnomaly,
    
    // Random/Quantum
    EntropyAnomaly,
    QRNGDeviation,
    
    // Multi-sensor
    CorrelatedAnomaly,
    SensorFusionEvent,
    
    // Unclassified
    Unknown,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Classification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub category: String,
    pub subcategory: Option<String>,
    pub confidence: f64,
    pub model_version: String,
}

/// Main detection engine
pub struct DetectionEngine {
    config: Arc<Config>,
    fusion_engine: FusionEngine,
    classifier: AnomalyClassifier,
    correlator: parking_lot::Mutex<SensorCorrelator>,
    event_bus: Arc<EventBus>,
    
    // Detection state
    recent_detections: RwLock<Vec<Detection>>,
    detection_count: RwLock<usize>,
}

impl DetectionEngine {
    pub async fn new(config: Arc<Config>, event_bus: Arc<EventBus>) -> Result<Self> {
        Ok(Self {
            config,
            fusion_engine: FusionEngine::new(),
            classifier: AnomalyClassifier::new(),
            correlator: parking_lot::Mutex::new(SensorCorrelator::new()),
            event_bus,
            recent_detections: RwLock::new(Vec::new()),
            detection_count: RwLock::new(0),
        })
    }
    
    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        info!("Starting detection engine...");
        
        // Subscribe to readings
        let mut reading_rx = self.event_bus.subscribe_readings();
        
        loop {
            tokio::select! {
                Ok(reading) = reading_rx.recv() => {
                    if let Some(detection) = self.process_reading(&reading).await {
                        self.record_detection(detection).await;
                    }
                }
                _ = shutdown.recv() => {
                    info!("Detection engine shutting down...");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn process_reading(&self, reading: &SensorReading) -> Option<Detection> {
        // Add to correlator for cross-sensor analysis
        self.correlator.lock().add_reading(reading.clone());
        
        // Check for correlated events
        if let Some(correlated) = self.correlator.lock().check_correlation() {
            let detection = self.create_detection(
                DetectionType::CorrelatedAnomaly,
                correlated.confidence,
                correlated.sensors,
            );
            return Some(detection);
        }
        
        None
    }
    
    fn create_detection(
        &self,
        detection_type: DetectionType,
        confidence: f64,
        sensors: Vec<SensorContribution>,
    ) -> Detection {
        let severity = match confidence {
            c if c >= 0.9 => Severity::Critical,
            c if c >= 0.7 => Severity::High,
            c if c >= 0.4 => Severity::Medium,
            _ => Severity::Low,
        };
        
        Detection {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            detection_type,
            confidence,
            severity,
            sensors,
            entropy_deviation: 0.0,
            anomaly_count: 0,
            correlation_score: 0.0,
            classification: None,
            location: None,
            data_window_start: Utc::now(),
            data_window_end: Utc::now(),
        }
    }
    
    async fn record_detection(&self, detection: Detection) {
        // Increment count
        {
            let mut count = self.detection_count.write().await;
            *count += 1;
        }
        
        // Store in recent
        {
            let mut recent = self.recent_detections.write().await;
            recent.push(detection.clone());
            
            // Keep only last 1000
            if recent.len() > 1000 {
                let drain_count = recent.len() - 1000; recent.drain(0..drain_count);
            }
        }
        
        // Publish event
        self.event_bus.publish_detection(detection);
    }
    
    pub async fn get_detection_count(&self) -> usize {
        *self.detection_count.read().await
    }
    
    pub async fn get_recent_detections(&self, limit: usize) -> Vec<Detection> {
        let recent = self.recent_detections.read().await;
        recent.iter().rev().take(limit).cloned().collect()
    }
}
