// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Anomaly classification

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{Detection, DetectionType, SensorContribution};
use crate::analysis::EntropyResult;

/// Classification categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationCategory {
    pub name: String,
    pub description: String,
    pub typical_signatures: Vec<String>,
}

/// Anomaly classifier
pub struct AnomalyClassifier {
    categories: Vec<ClassificationCategory>,
    feature_weights: HashMap<String, f64>,
}

impl AnomalyClassifier {
    pub fn new() -> Self {
        let categories = vec![
            ClassificationCategory {
                name: "Natural".to_string(),
                description: "Natural environmental phenomena".to_string(),
                typical_signatures: vec![
                    "gradual temperature change".to_string(),
                    "seismic microactivity".to_string(),
                    "atmospheric pressure variation".to_string(),
                ],
            },
            ClassificationCategory {
                name: "Electronic".to_string(),
                description: "Electronic interference or malfunction".to_string(),
                typical_signatures: vec![
                    "60Hz EMF".to_string(),
                    "radio frequency burst".to_string(),
                    "power line interference".to_string(),
                ],
            },
            ClassificationCategory {
                name: "Human".to_string(),
                description: "Human-caused activity".to_string(),
                typical_signatures: vec![
                    "footstep vibration".to_string(),
                    "voice frequency".to_string(),
                    "body heat signature".to_string(),
                ],
            },
            ClassificationCategory {
                name: "Biological".to_string(),
                description: "Non-human biological activity".to_string(),
                typical_signatures: vec![
                    "small animal movement".to_string(),
                    "insect ultrasonic".to_string(),
                    "rodent activity".to_string(),
                ],
            },
            ClassificationCategory {
                name: "Unexplained".to_string(),
                description: "Anomaly with no clear natural explanation".to_string(),
                typical_signatures: vec![
                    "sudden temperature drop".to_string(),
                    "correlated multi-sensor event".to_string(),
                    "entropy deviation".to_string(),
                    "non-periodic EMF".to_string(),
                ],
            },
        ];
        
        Self {
            categories,
            feature_weights: HashMap::new(),
        }
    }
    
    /// Classify a detection
    pub fn classify(&self, detection: &Detection) -> ClassificationResult {
        let features = self.extract_features(detection);
        let scores = self.score_categories(&features);
        
        let (best_category, best_score) = scores.iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(c, s)| (c.clone(), *s))
            .unwrap_or(("Unknown".to_string(), 0.0));
        
        ClassificationResult {
            category: best_category,
            confidence: best_score,
            all_scores: scores,
            features,
        }
    }
    
    fn extract_features(&self, detection: &Detection) -> HashMap<String, f64> {
        let mut features = HashMap::new();
        
        // Detection type features
        features.insert("is_thermal".to_string(), 
            if matches!(detection.detection_type, 
                DetectionType::ThermalAnomaly | DetectionType::ColdSpot | DetectionType::HotSpot) 
            { 1.0 } else { 0.0 });
        
        features.insert("is_emf".to_string(),
            if matches!(detection.detection_type,
                DetectionType::EMFSpike | DetectionType::EMFFluctuation | DetectionType::MagneticAnomaly)
            { 1.0 } else { 0.0 });
        
        features.insert("is_acoustic".to_string(),
            if matches!(detection.detection_type,
                DetectionType::InfrasoundEvent | DetectionType::UltrasonicEvent | DetectionType::UnexplainedSound)
            { 1.0 } else { 0.0 });
        
        features.insert("is_seismic".to_string(),
            if matches!(detection.detection_type,
                DetectionType::SeismicEvent | DetectionType::Vibration | DetectionType::Movement)
            { 1.0 } else { 0.0 });
        
        // Sensor count
        features.insert("sensor_count".to_string(), detection.sensors.len() as f64);
        
        // Correlation
        features.insert("correlation".to_string(), detection.correlation_score);
        
        // Entropy deviation
        features.insert("entropy_deviation".to_string(), detection.entropy_deviation);
        
        // Confidence
        features.insert("confidence".to_string(), detection.confidence);
        
        // Multi-sensor indicator
        features.insert("multi_sensor".to_string(),
            if detection.sensors.len() > 2 { 1.0 } else { 0.0 });
        
        features
    }
    
    fn score_categories(&self, features: &HashMap<String, f64>) -> HashMap<String, f64> {
        let mut scores = HashMap::new();
        
        // Natural phenomena scoring
        let natural_score = {
            let seismic = features.get("is_seismic").copied().unwrap_or(0.0);
            let low_corr = 1.0 - features.get("correlation").copied().unwrap_or(0.0);
            let low_entropy = 1.0 - features.get("entropy_deviation").copied().unwrap_or(0.0);
            (seismic * 0.3 + low_corr * 0.3 + low_entropy * 0.4).min(1.0)
        };
        scores.insert("Natural".to_string(), natural_score);
        
        // Electronic interference scoring
        let electronic_score = {
            let emf = features.get("is_emf").copied().unwrap_or(0.0);
            let single_sensor = if features.get("sensor_count").copied().unwrap_or(0.0) <= 1.0 { 0.5 } else { 0.0 };
            (emf * 0.6 + single_sensor * 0.4).min(1.0)
        };
        scores.insert("Electronic".to_string(), electronic_score);
        
        // Human activity scoring
        let human_score = {
            let thermal = features.get("is_thermal").copied().unwrap_or(0.0);
            let acoustic = features.get("is_acoustic").copied().unwrap_or(0.0);
            let seismic = features.get("is_seismic").copied().unwrap_or(0.0);
            ((thermal + acoustic + seismic) / 3.0 * 0.7).min(1.0)
        };
        scores.insert("Human".to_string(), human_score);
        
        // Biological scoring
        let biological_score = {
            let ultrasonic = if matches!(features.get("is_acoustic"), Some(&v) if v > 0.5) { 0.3 } else { 0.0 };
            let seismic = features.get("is_seismic").copied().unwrap_or(0.0) * 0.3;
            let low_thermal = (1.0 - features.get("is_thermal").copied().unwrap_or(0.0)) * 0.2;
            (ultrasonic + seismic + low_thermal).min(1.0)
        };
        scores.insert("Biological".to_string(), biological_score);
        
        // Unexplained scoring
        let unexplained_score = {
            let high_corr = features.get("correlation").copied().unwrap_or(0.0);
            let high_entropy = features.get("entropy_deviation").copied().unwrap_or(0.0);
            let multi_sensor = features.get("multi_sensor").copied().unwrap_or(0.0);
            let confidence = features.get("confidence").copied().unwrap_or(0.0);
            
            // Unexplained if high correlation across multiple sensors with entropy anomaly
            ((high_corr * 0.3 + high_entropy * 0.3 + multi_sensor * 0.2 + confidence * 0.2) * 1.2).min(1.0)
        };
        scores.insert("Unexplained".to_string(), unexplained_score);
        
        // Normalize
        let total: f64 = scores.values().sum();
        if total > 0.0 {
            for v in scores.values_mut() {
                *v /= total;
            }
        }
        
        scores
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationResult {
    pub category: String,
    pub confidence: f64,
    pub all_scores: HashMap<String, f64>,
    pub features: HashMap<String, f64>,
}
