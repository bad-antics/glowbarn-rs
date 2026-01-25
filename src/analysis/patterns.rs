// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Pattern detection - recurring patterns, periodicity, correlations

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::AnalysisConfig;

/// Detected pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern_type: PatternType,
    pub start_index: usize,
    pub length: usize,
    pub confidence: f64,
    pub period: Option<f64>,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    Periodic,
    Transient,
    Trend,
    Oscillation,
    StepChange,
    Impulse,
    Harmonic,
    Burst,
    Recurring,
}

/// Pattern detector
pub struct PatternDetector {
    config: AnalysisConfig,
}

impl PatternDetector {
    pub fn new(config: AnalysisConfig) -> Self {
        Self { config }
    }
    
    pub fn find_patterns(&self, data: &[f64]) -> Vec<Pattern> {
        let mut patterns = Vec::new();
        
        if data.len() < self.config.pattern_min_length {
            return patterns;
        }
        
        // Detect periodicity
        if let Some(period_pattern) = self.detect_periodicity(data) {
            patterns.push(period_pattern);
        }
        
        // Detect transients
        patterns.extend(self.detect_transients(data));
        
        // Detect trends
        if let Some(trend_pattern) = self.detect_trend(data) {
            patterns.push(trend_pattern);
        }
        
        // Detect step changes
        patterns.extend(self.detect_step_changes(data));
        
        // Detect recurring subsequences
        patterns.extend(self.detect_recurring_motifs(data));
        
        patterns
    }
    
    /// Detect periodicity using autocorrelation
    fn detect_periodicity(&self, data: &[f64]) -> Option<Pattern> {
        let n = data.len();
        if n < 32 {
            return None;
        }
        
        // Compute autocorrelation
        let mean = data.iter().sum::<f64>() / n as f64;
        let variance: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum();
        
        if variance < 1e-10 {
            return None;
        }
        
        let max_lag = n / 2;
        let mut autocorr = Vec::with_capacity(max_lag);
        
        for lag in 0..max_lag {
            let mut sum = 0.0;
            for i in 0..(n - lag) {
                sum += (data[i] - mean) * (data[i + lag] - mean);
            }
            autocorr.push(sum / variance);
        }
        
        // Find first significant peak after lag 0
        let threshold = 0.3;
        let mut in_trough = false;
        
        for lag in 1..max_lag {
            if autocorr[lag] < threshold {
                in_trough = true;
            }
            if in_trough && autocorr[lag] > threshold {
                // Found period
                let confidence = autocorr[lag].min(1.0);
                
                return Some(Pattern {
                    pattern_type: PatternType::Periodic,
                    start_index: 0,
                    length: n,
                    confidence,
                    period: Some(lag as f64),
                    description: format!("Periodic signal with period {}", lag),
                });
            }
        }
        
        None
    }
    
    /// Detect transient events
    fn detect_transients(&self, data: &[f64]) -> Vec<Pattern> {
        let mut patterns = Vec::new();
        
        if data.len() < 10 {
            return patterns;
        }
        
        // Compute short-term energy
        let window = 5;
        let energies: Vec<f64> = data.windows(window)
            .map(|w| w.iter().map(|&x| x * x).sum::<f64>() / window as f64)
            .collect();
        
        // Compute threshold
        let mean_energy = energies.iter().sum::<f64>() / energies.len() as f64;
        let std_energy = (energies.iter().map(|&e| (e - mean_energy).powi(2)).sum::<f64>() 
            / energies.len() as f64).sqrt();
        let threshold = mean_energy + 3.0 * std_energy;
        
        // Find transients
        let mut in_transient = false;
        let mut start = 0;
        
        for (i, &energy) in energies.iter().enumerate() {
            if energy > threshold && !in_transient {
                in_transient = true;
                start = i;
            } else if energy <= threshold && in_transient {
                in_transient = false;
                let length = i - start;
                if length >= 3 {
                    patterns.push(Pattern {
                        pattern_type: PatternType::Transient,
                        start_index: start,
                        length,
                        confidence: ((energies[start..i].iter().cloned().fold(f64::MIN, f64::max) - threshold) 
                            / threshold).min(1.0),
                        period: None,
                        description: format!("Transient at index {} with length {}", start, length),
                    });
                }
            }
        }
        
        patterns
    }
    
    /// Detect overall trend
    fn detect_trend(&self, data: &[f64]) -> Option<Pattern> {
        if data.len() < 20 {
            return None;
        }
        
        // Linear regression
        let n = data.len() as f64;
        let sum_x: f64 = (0..data.len()).map(|i| i as f64).sum();
        let sum_y: f64 = data.iter().sum();
        let sum_xy: f64 = data.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
        let sum_xx: f64 = (0..data.len()).map(|i| (i * i) as f64).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;
        
        // R-squared
        let mean_y = sum_y / n;
        let ss_tot: f64 = data.iter().map(|&y| (y - mean_y).powi(2)).sum();
        let ss_res: f64 = data.iter().enumerate()
            .map(|(i, &y)| (y - (slope * i as f64 + intercept)).powi(2))
            .sum();
        
        let r_squared = if ss_tot > 1e-10 { 1.0 - ss_res / ss_tot } else { 0.0 };
        
        // Only report significant trends
        if r_squared > 0.3 && slope.abs() > 1e-6 {
            let direction = if slope > 0.0 { "upward" } else { "downward" };
            Some(Pattern {
                pattern_type: PatternType::Trend,
                start_index: 0,
                length: data.len(),
                confidence: r_squared,
                period: None,
                description: format!("Linear {} trend (R²={:.2})", direction, r_squared),
            })
        } else {
            None
        }
    }
    
    /// Detect step changes using CUSUM
    fn detect_step_changes(&self, data: &[f64]) -> Vec<Pattern> {
        let mut patterns = Vec::new();
        
        if data.len() < 20 {
            return patterns;
        }
        
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std = (data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() 
            / (data.len() - 1) as f64).sqrt();
        
        if std < 1e-10 {
            return patterns;
        }
        
        // Sliding window comparison
        let window = data.len() / 10;
        if window < 5 {
            return patterns;
        }
        
        for i in window..(data.len() - window) {
            let left_mean = data[i-window..i].iter().sum::<f64>() / window as f64;
            let right_mean = data[i..i+window].iter().sum::<f64>() / window as f64;
            
            let diff = (right_mean - left_mean).abs();
            if diff > 2.0 * std {
                // Avoid duplicates
                if patterns.last().map(|p: &Pattern| i - p.start_index > window).unwrap_or(true) {
                    patterns.push(Pattern {
                        pattern_type: PatternType::StepChange,
                        start_index: i,
                        length: 1,
                        confidence: (diff / std / 3.0).min(1.0),
                        period: None,
                        description: format!("Step change at index {} ({:.2} σ)", i, diff / std),
                    });
                }
            }
        }
        
        patterns
    }
    
    /// Detect recurring motifs using Matrix Profile (simplified)
    fn detect_recurring_motifs(&self, data: &[f64]) -> Vec<Pattern> {
        let mut patterns = Vec::new();
        
        let motif_length = self.config.pattern_min_length;
        if data.len() < motif_length * 3 {
            return patterns;
        }
        
        // Simplified Matrix Profile
        let n_subsequences = data.len() - motif_length + 1;
        let mut min_distances = vec![f64::MAX; n_subsequences];
        let mut nearest_neighbor = vec![0usize; n_subsequences];
        
        for i in 0..n_subsequences {
            let subseq_i = &data[i..i+motif_length];
            let mean_i = subseq_i.iter().sum::<f64>() / motif_length as f64;
            let std_i = (subseq_i.iter().map(|&x| (x - mean_i).powi(2)).sum::<f64>() 
                / motif_length as f64).sqrt();
            
            for j in (i + motif_length)..n_subsequences {
                let subseq_j = &data[j..j+motif_length];
                let mean_j = subseq_j.iter().sum::<f64>() / motif_length as f64;
                let std_j = (subseq_j.iter().map(|&x| (x - mean_j).powi(2)).sum::<f64>() 
                    / motif_length as f64).sqrt();
                
                // Z-normalized Euclidean distance
                if std_i > 1e-10 && std_j > 1e-10 {
                    let dist: f64 = subseq_i.iter().zip(subseq_j.iter())
                        .map(|(&a, &b)| {
                            let za = (a - mean_i) / std_i;
                            let zb = (b - mean_j) / std_j;
                            (za - zb).powi(2)
                        })
                        .sum::<f64>().sqrt();
                    
                    if dist < min_distances[i] {
                        min_distances[i] = dist;
                        nearest_neighbor[i] = j;
                    }
                    if dist < min_distances[j] {
                        min_distances[j] = dist;
                        nearest_neighbor[j] = i;
                    }
                }
            }
        }
        
        // Find motif pairs (low distance = similar patterns)
        let threshold = 0.5;  // Normalized distance threshold
        let mut motif_indices: Vec<(usize, f64)> = min_distances.iter()
            .enumerate()
            .filter(|(_, &d)| d < threshold * (motif_length as f64).sqrt())
            .map(|(i, &d)| (i, d))
            .collect();
        
        motif_indices.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Report top motifs
        let mut reported = std::collections::HashSet::new();
        for (idx, dist) in motif_indices.into_iter().take(3) {
            if reported.contains(&idx) || reported.contains(&nearest_neighbor[idx]) {
                continue;
            }
            reported.insert(idx);
            reported.insert(nearest_neighbor[idx]);
            
            patterns.push(Pattern {
                pattern_type: PatternType::Recurring,
                start_index: idx,
                length: motif_length,
                confidence: 1.0 - dist / (threshold * (motif_length as f64).sqrt()),
                period: Some((nearest_neighbor[idx] - idx) as f64),
                description: format!("Recurring motif at {} and {}", idx, nearest_neighbor[idx]),
            });
        }
        
        patterns
    }
}
