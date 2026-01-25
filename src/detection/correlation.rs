//! Sensor correlation analysis

use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::sensors::{SensorReading, SensorType};
use super::SensorContribution;

/// Correlation event detected
#[derive(Debug, Clone)]
pub struct CorrelationEvent {
    pub timestamp: DateTime<Utc>,
    pub sensors: Vec<SensorContribution>,
    pub confidence: f64,
    pub lag_ms: i64,
}

/// Sensor correlator
pub struct SensorCorrelator {
    // Buffer of recent readings per sensor
    buffers: HashMap<String, VecDeque<TimestampedReading>>,
    buffer_duration_ms: i64,
    min_correlation: f64,
    
    // Correlation windows
    correlation_window_ms: i64,
}

#[derive(Debug, Clone)]
struct TimestampedReading {
    timestamp: DateTime<Utc>,
    value: f64,
    sensor_type: SensorType,
    anomaly_score: f64,
}

impl SensorCorrelator {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            buffer_duration_ms: 10000,  // 10 seconds
            min_correlation: 0.5,
            correlation_window_ms: 2000,  // 2 second window
        }
    }
    
    /// Add a reading to correlation tracking
    pub fn add_reading(&mut self, reading: SensorReading) {
        let value = if reading.data.is_empty() {
            0.0
        } else {
            reading.data.iter().sum::<f64>() / reading.data.len() as f64
        };
        
        let anomaly_score = self.quick_anomaly_score(&reading);
        
        let entry = TimestampedReading {
            timestamp: reading.timestamp,
            value,
            sensor_type: reading.sensor_type,
            anomaly_score,
        };
        
        let buffer = self.buffers
            .entry(reading.sensor_id)
            .or_insert_with(VecDeque::new);
        
        buffer.push_back(entry);
        
        // Remove old entries
        let cutoff = Utc::now() - Duration::milliseconds(self.buffer_duration_ms);
        while buffer.front().map(|r| r.timestamp < cutoff).unwrap_or(false) {
            buffer.pop_front();
        }
    }
    
    /// Check for correlated events across sensors
    pub fn check_correlation(&self) -> Option<CorrelationEvent> {
        let now = Utc::now();
        let window_start = now - Duration::milliseconds(self.correlation_window_ms);
        
        // Collect recent anomalous readings from different sensors
        let mut anomalous_readings: Vec<(&String, &TimestampedReading)> = Vec::new();
        
        for (sensor_id, buffer) in &self.buffers {
            for reading in buffer.iter().rev() {
                if reading.timestamp < window_start {
                    break;
                }
                
                if reading.anomaly_score > 0.3 {
                    anomalous_readings.push((sensor_id, reading));
                }
            }
        }
        
        // Need at least 2 different sensors with anomalies
        let unique_sensors: std::collections::HashSet<_> = anomalous_readings.iter()
            .map(|(id, _)| *id)
            .collect();
        
        if unique_sensors.len() < 2 {
            return None;
        }
        
        // Calculate correlation score
        let sensor_contributions: Vec<SensorContribution> = anomalous_readings.iter()
            .map(|(sensor_id, reading)| {
                SensorContribution {
                    sensor_id: (*sensor_id).clone(),
                    sensor_type: reading.sensor_type,
                    weight: self.get_sensor_weight(reading.sensor_type),
                    reading_value: reading.value,
                    anomaly_score: reading.anomaly_score,
                }
            })
            .collect();
        
        // Calculate overall confidence
        let avg_anomaly: f64 = sensor_contributions.iter()
            .map(|s| s.anomaly_score)
            .sum::<f64>() / sensor_contributions.len() as f64;
        
        let sensor_diversity = unique_sensors.len() as f64 / 5.0;  // Normalize
        
        let confidence = (avg_anomaly * 0.6 + sensor_diversity * 0.4).min(1.0);
        
        if confidence > self.min_correlation {
            // Calculate time lag between first and last anomaly
            let timestamps: Vec<_> = anomalous_readings.iter()
                .map(|(_, r)| r.timestamp)
                .collect();
            let min_time = timestamps.iter().min()?;
            let max_time = timestamps.iter().max()?;
            let lag_ms = (*max_time - *min_time).num_milliseconds();
            
            Some(CorrelationEvent {
                timestamp: now,
                sensors: sensor_contributions,
                confidence,
                lag_ms,
            })
        } else {
            None
        }
    }
    
    /// Calculate cross-correlation between two sensor buffers
    pub fn cross_correlate(&self, sensor1: &str, sensor2: &str, max_lag_ms: i64) -> Option<(f64, i64)> {
        let buffer1 = self.buffers.get(sensor1)?;
        let buffer2 = self.buffers.get(sensor2)?;
        
        if buffer1.len() < 10 || buffer2.len() < 10 {
            return None;
        }
        
        let values1: Vec<f64> = buffer1.iter().map(|r| r.value).collect();
        let values2: Vec<f64> = buffer2.iter().map(|r| r.value).collect();
        
        let mean1 = values1.iter().sum::<f64>() / values1.len() as f64;
        let mean2 = values2.iter().sum::<f64>() / values2.len() as f64;
        
        let std1 = (values1.iter().map(|&x| (x - mean1).powi(2)).sum::<f64>() 
            / values1.len() as f64).sqrt();
        let std2 = (values2.iter().map(|&x| (x - mean2).powi(2)).sum::<f64>() 
            / values2.len() as f64).sqrt();
        
        if std1 < 1e-10 || std2 < 1e-10 {
            return None;
        }
        
        let n = values1.len().min(values2.len());
        let max_lag = (max_lag_ms / 100) as usize;  // Assuming ~100ms between readings
        
        let mut best_corr = 0.0_f64;
        let mut best_lag: i64 = 0;
        
        for lag in 0..max_lag.min(n/2) {
            // Positive lag (sensor2 leads)
            let corr = self.compute_correlation(&values1[lag..], &values2[..n-lag], mean1, mean2, std1, std2);
            if corr.abs() > best_corr.abs() {
                best_corr = corr;
                best_lag = (lag as i64) * 100;  // Convert to ms
            }
            
            // Negative lag (sensor1 leads)
            let corr = self.compute_correlation(&values1[..n-lag], &values2[lag..], mean1, mean2, std1, std2);
            if corr.abs() > best_corr.abs() {
                best_corr = corr;
                best_lag = -(lag as i64) * 100;
            }
        }
        
        Some((best_corr, best_lag))
    }
    
    fn compute_correlation(&self, v1: &[f64], v2: &[f64], mean1: f64, mean2: f64, std1: f64, std2: f64) -> f64 {
        let n = v1.len().min(v2.len());
        if n == 0 {
            return 0.0;
        }
        
        let sum: f64 = v1.iter().zip(v2.iter())
            .map(|(&a, &b)| (a - mean1) * (b - mean2))
            .sum();
        
        sum / (n as f64 * std1 * std2)
    }
    
    fn quick_anomaly_score(&self, reading: &SensorReading) -> f64 {
        if reading.data.is_empty() {
            return 0.0;
        }
        
        let mean = reading.data.iter().sum::<f64>() / reading.data.len() as f64;
        let variance = reading.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / reading.data.len() as f64;
        let std_dev = variance.sqrt();
        
        if std_dev < 1e-10 {
            return 0.0;
        }
        
        let max_deviation = reading.data.iter()
            .map(|&x| (x - mean).abs())
            .fold(0.0_f64, f64::max);
        
        let z_score = max_deviation / std_dev;
        
        // Convert to 0-1 score
        1.0 / (1.0 + (-0.5 * (z_score - 2.0)).exp())
    }
    
    fn get_sensor_weight(&self, sensor_type: SensorType) -> f64 {
        match sensor_type {
            SensorType::GeigerCounter | SensorType::LaserGrid => 0.95,
            SensorType::ThermalImager | SensorType::FluxGate => 0.85,
            SensorType::ThermalArray | SensorType::Geophone => 0.80,
            SensorType::Accelerometer | SensorType::Infrasound | SensorType::Ultrasonic => 0.75,
            SensorType::EMFProbe | SensorType::SDRReceiver => 0.70,
            _ => 0.60,
        }
    }
    
    /// Get correlation matrix for all active sensors
    pub fn get_correlation_matrix(&self) -> HashMap<(String, String), f64> {
        let mut matrix = HashMap::new();
        
        let sensor_ids: Vec<_> = self.buffers.keys().cloned().collect();
        
        for i in 0..sensor_ids.len() {
            for j in (i+1)..sensor_ids.len() {
                if let Some((corr, _)) = self.cross_correlate(&sensor_ids[i], &sensor_ids[j], 2000) {
                    matrix.insert((sensor_ids[i].clone(), sensor_ids[j].clone()), corr);
                }
            }
        }
        
        matrix
    }
}
