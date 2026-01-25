//! Analysis module - entropy, anomaly detection, signal processing

mod entropy;
mod anomaly;
mod signal;
mod patterns;
mod statistics;
mod complexity;

pub use entropy::*;
pub use anomaly::*;
pub use signal::*;
pub use patterns::*;
pub use statistics::*;
pub use complexity::*;

use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use anyhow::Result;
use tracing::{info, debug};

use crate::sensors::SensorReading;
use crate::config::Config;
use crate::core::EventBus;

/// Analysis engine configuration
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub entropy_window: usize,
    pub anomaly_threshold: f64,
    pub pattern_min_length: usize,
    pub fft_size: usize,
    pub enable_gpu: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            entropy_window: 1024,
            anomaly_threshold: 3.0,  // Standard deviations
            pattern_min_length: 16,
            fft_size: 4096,
            enable_gpu: true,
        }
    }
}

/// Main analysis engine
pub struct AnalysisEngine {
    config: Arc<Config>,
    analysis_config: AnalysisConfig,
    entropy_analyzer: EntropyAnalyzer,
    anomaly_detector: AnomalyDetector,
    signal_processor: SignalProcessor,
    pattern_detector: PatternDetector,
    event_bus: Arc<EventBus>,
}

impl AnalysisEngine {
    pub async fn new(config: Arc<Config>, event_bus: Arc<EventBus>) -> Result<Self> {
        let analysis_config = AnalysisConfig::default();
        
        Ok(Self {
            config,
            analysis_config: analysis_config.clone(),
            entropy_analyzer: EntropyAnalyzer::new(analysis_config.clone()),
            anomaly_detector: AnomalyDetector::new(analysis_config.clone()),
            signal_processor: SignalProcessor::new(analysis_config.clone()),
            pattern_detector: PatternDetector::new(analysis_config.clone()),
            event_bus,
        })
    }
    
    pub async fn run(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        info!("Starting analysis engine...");
        
        let mut reading_rx = self.event_bus.subscribe_readings();
        
        loop {
            tokio::select! {
                Ok(reading) = reading_rx.recv() => {
                    self.process_reading(&reading).await;
                }
                _ = shutdown.recv() => {
                    info!("Analysis engine shutting down...");
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    async fn process_reading(&self, reading: &SensorReading) {
        if reading.data.is_empty() {
            return;
        }
        
        // Compute entropy metrics
        let entropy_result = self.entropy_analyzer.analyze(&reading.data);
        
        // Detect anomalies
        let anomalies = self.anomaly_detector.detect(&reading.data);
        
        // Signal analysis
        let signal_features = self.signal_processor.extract_features(&reading.data, reading.sample_rate);
        
        // Pattern detection
        let patterns = self.pattern_detector.find_patterns(&reading.data);
        
        // Publish results
        if !anomalies.is_empty() || entropy_result.is_anomalous {
            debug!("Anomaly detected in {}: entropy={:.4}, anomalies={}",
                reading.sensor_id, entropy_result.shannon, anomalies.len());
        }
    }
}
