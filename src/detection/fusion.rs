// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Sensor fusion engine - Bayesian, Dempster-Shafer, and neural fusion

use std::collections::HashMap;
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

use crate::sensors::{SensorReading, SensorType};
use super::{SensorContribution, DetectionType};

/// Fusion result
#[derive(Debug, Clone)]
pub struct FusionResult {
    pub confidence: f64,
    pub detection_type: DetectionType,
    pub sensors: Vec<SensorContribution>,
    pub belief_mass: HashMap<String, f64>,
}

/// Sensor fusion engine
pub struct FusionEngine {
    // Sensor reliability weights
    sensor_weights: HashMap<SensorType, f64>,
    
    // Belief masses for Dempster-Shafer
    belief_masses: HashMap<SensorType, BeliefMass>,
    
    // Recent readings for temporal fusion
    reading_buffer: HashMap<String, Vec<SensorReading>>,
    buffer_size: usize,
}

/// Dempster-Shafer belief mass
#[derive(Debug, Clone, Default)]
pub struct BeliefMass {
    pub anomaly: f64,      // m({anomaly})
    pub normal: f64,       // m({normal})
    pub uncertainty: f64,  // m(Θ) - complete uncertainty
}

impl FusionEngine {
    pub fn new() -> Self {
        let mut sensor_weights = HashMap::new();
        
        // Default sensor reliability weights
        sensor_weights.insert(SensorType::ThermalImager, 0.85);
        sensor_weights.insert(SensorType::ThermalArray, 0.80);
        sensor_weights.insert(SensorType::Accelerometer, 0.75);
        sensor_weights.insert(SensorType::Geophone, 0.80);
        sensor_weights.insert(SensorType::EMFProbe, 0.70);
        sensor_weights.insert(SensorType::FluxGate, 0.85);
        sensor_weights.insert(SensorType::Infrasound, 0.75);
        sensor_weights.insert(SensorType::Ultrasonic, 0.75);
        sensor_weights.insert(SensorType::GeigerCounter, 0.90);
        sensor_weights.insert(SensorType::QRNG, 0.80);
        sensor_weights.insert(SensorType::SDRReceiver, 0.70);
        sensor_weights.insert(SensorType::LaserGrid, 0.95);
        
        Self {
            sensor_weights,
            belief_masses: HashMap::new(),
            reading_buffer: HashMap::new(),
            buffer_size: 100,
        }
    }
    
    /// Add reading to fusion buffer
    pub fn add_reading(&mut self, reading: SensorReading) {
        let buffer = self.reading_buffer
            .entry(reading.sensor_id.clone())
            .or_insert_with(Vec::new);
        
        buffer.push(reading);
        
        if buffer.len() > self.buffer_size {
            buffer.drain(0..buffer.len() - self.buffer_size);
        }
    }
    
    /// Bayesian fusion of multiple sensor readings
    pub fn bayesian_fusion(&self, readings: &[SensorReading], prior_anomaly: f64) -> FusionResult {
        if readings.is_empty() {
            return FusionResult {
                confidence: 0.0,
                detection_type: DetectionType::Unknown,
                sensors: vec![],
                belief_mass: HashMap::new(),
            };
        }
        
        // Start with prior probability
        let mut posterior = prior_anomaly;
        let mut sensors = Vec::new();
        
        for reading in readings {
            let weight = self.sensor_weights
                .get(&reading.sensor_type)
                .copied()
                .unwrap_or(0.5);
            
            // Calculate likelihood based on reading properties
            let anomaly_score = self.calculate_anomaly_score(reading);
            
            // P(anomaly | evidence) ∝ P(evidence | anomaly) * P(anomaly)
            let likelihood_anomaly = anomaly_score;
            let likelihood_normal = 1.0 - anomaly_score;
            
            // Bayes update
            let evidence = likelihood_anomaly * posterior + likelihood_normal * (1.0 - posterior);
            if evidence > 1e-10 {
                posterior = (likelihood_anomaly * posterior) / evidence;
            }
            
            // Weight by sensor reliability
            posterior = posterior * weight + prior_anomaly * (1.0 - weight);
            
            sensors.push(SensorContribution {
                sensor_id: reading.sensor_id.clone(),
                sensor_type: reading.sensor_type,
                weight,
                reading_value: reading.data.iter().sum::<f64>() / reading.data.len().max(1) as f64,
                anomaly_score,
            });
        }
        
        let detection_type = self.classify_from_sensors(&sensors);
        
        FusionResult {
            confidence: posterior,
            detection_type,
            sensors,
            belief_mass: HashMap::new(),
        }
    }
    
    /// Dempster-Shafer fusion for handling uncertainty
    pub fn dempster_shafer_fusion(&self, readings: &[SensorReading]) -> FusionResult {
        if readings.is_empty() {
            return FusionResult {
                confidence: 0.0,
                detection_type: DetectionType::Unknown,
                sensors: vec![],
                belief_mass: HashMap::new(),
            };
        }
        
        // Initialize with complete uncertainty
        let mut combined = BeliefMass {
            anomaly: 0.0,
            normal: 0.0,
            uncertainty: 1.0,
        };
        
        let mut sensors = Vec::new();
        
        for reading in readings {
            let anomaly_score = self.calculate_anomaly_score(reading);
            let weight = self.sensor_weights
                .get(&reading.sensor_type)
                .copied()
                .unwrap_or(0.5);
            
            // Create belief mass for this reading
            let mass = BeliefMass {
                anomaly: anomaly_score * weight,
                normal: (1.0 - anomaly_score) * weight,
                uncertainty: 1.0 - weight,
            };
            
            // Dempster's rule of combination
            combined = self.combine_belief_masses(&combined, &mass);
            
            sensors.push(SensorContribution {
                sensor_id: reading.sensor_id.clone(),
                sensor_type: reading.sensor_type,
                weight,
                reading_value: reading.data.iter().sum::<f64>() / reading.data.len().max(1) as f64,
                anomaly_score,
            });
        }
        
        // Normalize (handle conflict)
        let total = combined.anomaly + combined.normal + combined.uncertainty;
        if total > 1e-10 {
            combined.anomaly /= total;
            combined.normal /= total;
            combined.uncertainty /= total;
        }
        
        let confidence = combined.anomaly / (combined.anomaly + combined.normal).max(1e-10);
        let detection_type = self.classify_from_sensors(&sensors);
        
        let mut belief_mass = HashMap::new();
        belief_mass.insert("anomaly".to_string(), combined.anomaly);
        belief_mass.insert("normal".to_string(), combined.normal);
        belief_mass.insert("uncertainty".to_string(), combined.uncertainty);
        
        FusionResult {
            confidence,
            detection_type,
            sensors,
            belief_mass,
        }
    }
    
    /// Combine two belief masses using Dempster's rule
    fn combine_belief_masses(&self, m1: &BeliefMass, m2: &BeliefMass) -> BeliefMass {
        // Calculate combined masses
        // m12(A) = Σ(m1(B) * m2(C)) for B∩C=A, divided by (1-K)
        // K is the conflict
        
        let k = m1.anomaly * m2.normal + m1.normal * m2.anomaly;  // Conflict
        let normalizer = (1.0 - k).max(1e-10);
        
        let anomaly = (
            m1.anomaly * m2.anomaly +
            m1.anomaly * m2.uncertainty +
            m1.uncertainty * m2.anomaly
        ) / normalizer;
        
        let normal = (
            m1.normal * m2.normal +
            m1.normal * m2.uncertainty +
            m1.uncertainty * m2.normal
        ) / normalizer;
        
        let uncertainty = (m1.uncertainty * m2.uncertainty) / normalizer;
        
        BeliefMass { anomaly, normal, uncertainty }
    }
    
    /// Weighted average fusion (simple but effective)
    pub fn weighted_fusion(&self, readings: &[SensorReading]) -> FusionResult {
        if readings.is_empty() {
            return FusionResult {
                confidence: 0.0,
                detection_type: DetectionType::Unknown,
                sensors: vec![],
                belief_mass: HashMap::new(),
            };
        }
        
        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;
        let mut sensors = Vec::new();
        
        for reading in readings {
            let weight = self.sensor_weights
                .get(&reading.sensor_type)
                .copied()
                .unwrap_or(0.5);
            
            let anomaly_score = self.calculate_anomaly_score(reading);
            
            weighted_sum += anomaly_score * weight;
            weight_sum += weight;
            
            sensors.push(SensorContribution {
                sensor_id: reading.sensor_id.clone(),
                sensor_type: reading.sensor_type,
                weight,
                reading_value: reading.data.iter().sum::<f64>() / reading.data.len().max(1) as f64,
                anomaly_score,
            });
        }
        
        let confidence = if weight_sum > 1e-10 {
            weighted_sum / weight_sum
        } else {
            0.0
        };
        
        let detection_type = self.classify_from_sensors(&sensors);
        
        FusionResult {
            confidence,
            detection_type,
            sensors,
            belief_mass: HashMap::new(),
        }
    }
    
    /// Calculate anomaly score for a reading
    fn calculate_anomaly_score(&self, reading: &SensorReading) -> f64 {
        if reading.data.is_empty() {
            return 0.0;
        }
        
        // Simple statistical anomaly score
        let mean = reading.data.iter().sum::<f64>() / reading.data.len() as f64;
        let variance = reading.data.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / reading.data.len() as f64;
        let std_dev = variance.sqrt();
        
        // Check for values far from mean
        let max_deviation = reading.data.iter()
            .map(|&x| (x - mean).abs())
            .fold(0.0_f64, f64::max);
        
        let z_score = if std_dev > 1e-10 {
            max_deviation / std_dev
        } else {
            0.0
        };
        
        // Sigmoid transform to [0, 1]
        let score = 1.0 / (1.0 + (-0.5 * (z_score - 2.0)).exp());
        
        // Adjust based on reading quality
        score * reading.quality as f64
    }
    
    /// Classify detection type from contributing sensors
    fn classify_from_sensors(&self, sensors: &[SensorContribution]) -> DetectionType {
        if sensors.is_empty() {
            return DetectionType::Unknown;
        }
        
        // Find dominant sensor type
        let max_sensor = sensors.iter()
            .max_by(|a, b| a.anomaly_score.partial_cmp(&b.anomaly_score).unwrap());
        
        match max_sensor.map(|s| s.sensor_type) {
            Some(SensorType::ThermalArray) | Some(SensorType::ThermalImager) => {
                DetectionType::ThermalAnomaly
            }
            Some(SensorType::Accelerometer) | Some(SensorType::Geophone) => {
                DetectionType::SeismicEvent
            }
            Some(SensorType::EMFProbe) | Some(SensorType::FluxGate) | Some(SensorType::TriField) => {
                DetectionType::EMFSpike
            }
            Some(SensorType::Infrasound) => DetectionType::InfrasoundEvent,
            Some(SensorType::Ultrasonic) => DetectionType::UltrasonicEvent,
            Some(SensorType::GeigerCounter) | Some(SensorType::Scintillator) => {
                DetectionType::RadiationSpike
            }
            Some(SensorType::QRNG) | Some(SensorType::ThermalNoise) => {
                DetectionType::EntropyAnomaly
            }
            Some(SensorType::SDRReceiver) | Some(SensorType::SpectrumAnalyzer) => {
                DetectionType::RFAnomaly
            }
            Some(SensorType::LaserGrid) => DetectionType::LaserInterruption,
            Some(SensorType::StaticMeter) => DetectionType::StaticDischarge,
            Some(SensorType::IonCounter) => DetectionType::IonizationChange,
            Some(SensorType::Spectrometer) | Some(SensorType::LightMeter) => {
                DetectionType::LightAnomaly
            }
            _ => DetectionType::Unknown,
        }
    }
    
    /// Set sensor weight
    pub fn set_sensor_weight(&mut self, sensor_type: SensorType, weight: f64) {
        self.sensor_weights.insert(sensor_type, weight.clamp(0.0, 1.0));
    }
    
    /// Get current sensor weights
    pub fn get_sensor_weights(&self) -> &HashMap<SensorType, f64> {
        &self.sensor_weights
    }
}
