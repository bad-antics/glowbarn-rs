//! Anomaly detection - statistical, ML, and ensemble methods

use std::collections::VecDeque;
use nalgebra::{DMatrix, DVector};
use serde::{Deserialize, Serialize};

use super::AnalysisConfig;

/// Detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub index: usize,
    pub value: f64,
    pub score: f64,
    pub anomaly_type: AnomalyType,
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    PointAnomaly,       // Single outlier
    ContextualAnomaly,  // Anomaly in context
    CollectiveAnomaly,  // Group of anomalies
    ChangePoint,        // Distribution shift
    Spike,              // Sudden spike
    Drop,               // Sudden drop
    Drift,              // Gradual change
    Oscillation,        // Abnormal oscillation
}

/// Anomaly detector with multiple methods
pub struct AnomalyDetector {
    config: AnalysisConfig,
    
    // Rolling statistics for adaptive detection
    history: VecDeque<f64>,
    history_size: usize,
    running_mean: f64,
    running_var: f64,
    
    // Isolation Forest state
    isolation_trees: Vec<IsolationTree>,
    
    // CUSUM parameters
    cusum_pos: f64,
    cusum_neg: f64,
}

impl AnomalyDetector {
    pub fn new(config: AnalysisConfig) -> Self {
        Self {
            config,
            history: VecDeque::with_capacity(10000),
            history_size: 10000,
            running_mean: 0.0,
            running_var: 1.0,
            isolation_trees: Vec::new(),
            cusum_pos: 0.0,
            cusum_neg: 0.0,
        }
    }
    
    pub fn detect(&self, data: &[f64]) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        
        // Statistical detection
        anomalies.extend(self.detect_statistical(data));
        
        // Isolation Forest
        anomalies.extend(self.detect_isolation_forest(data));
        
        // CUSUM for change detection
        anomalies.extend(self.detect_cusum(data));
        
        // Local Outlier Factor
        anomalies.extend(self.detect_lof(data));
        
        // Remove duplicates and sort by score
        self.deduplicate_anomalies(&mut anomalies);
        anomalies.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        anomalies
    }
    
    /// Statistical anomaly detection (Z-score, MAD, Grubbs)
    fn detect_statistical(&self, data: &[f64]) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        
        if data.len() < 10 {
            return anomalies;
        }
        
        // Z-score detection
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std = self.std_dev(data);
        
        if std > 1e-10 {
            for (i, &x) in data.iter().enumerate() {
                let z_score = (x - mean).abs() / std;
                if z_score > self.config.anomaly_threshold {
                    anomalies.push(Anomaly {
                        index: i,
                        value: x,
                        score: z_score,
                        anomaly_type: if x > mean { AnomalyType::Spike } else { AnomalyType::Drop },
                        confidence: self.z_score_to_confidence(z_score),
                    });
                }
            }
        }
        
        // MAD (Median Absolute Deviation) detection - more robust
        let median = self.median(data);
        let mad = self.mad(data, median);
        
        if mad > 1e-10 {
            let threshold = 3.5;  // Modified Z-score threshold
            for (i, &x) in data.iter().enumerate() {
                let modified_z = 0.6745 * (x - median) / mad;
                if modified_z.abs() > threshold {
                    // Only add if not already detected
                    if !anomalies.iter().any(|a| a.index == i) {
                        anomalies.push(Anomaly {
                            index: i,
                            value: x,
                            score: modified_z.abs(),
                            anomaly_type: AnomalyType::PointAnomaly,
                            confidence: self.z_score_to_confidence(modified_z.abs()),
                        });
                    }
                }
            }
        }
        
        anomalies
    }
    
    /// Isolation Forest anomaly detection
    fn detect_isolation_forest(&self, data: &[f64]) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        
        if data.len() < 100 {
            return anomalies;
        }
        
        let n_trees = 100;
        let sample_size = (data.len() / 4).min(256);
        
        // Build forest
        let trees: Vec<_> = (0..n_trees)
            .map(|_| IsolationTree::build(data, sample_size))
            .collect();
        
        // Score each point
        let avg_path_length = self.expected_path_length(sample_size);
        
        for (i, &x) in data.iter().enumerate() {
            let avg_depth: f64 = trees.iter()
                .map(|tree| tree.path_length(x) as f64)
                .sum::<f64>() / n_trees as f64;
            
            // Anomaly score: 2^(-avg_depth / avg_path_length)
            let score = 2.0_f64.powf(-avg_depth / avg_path_length);
            
            if score > 0.6 {  // Threshold for anomaly
                anomalies.push(Anomaly {
                    index: i,
                    value: x,
                    score: score * 10.0,  // Scale to be comparable
                    anomaly_type: AnomalyType::PointAnomaly,
                    confidence: score,
                });
            }
        }
        
        anomalies
    }
    
    /// CUSUM (Cumulative Sum) change point detection
    fn detect_cusum(&self, data: &[f64]) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        
        if data.len() < 20 {
            return anomalies;
        }
        
        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let std = self.std_dev(data);
        
        if std < 1e-10 {
            return anomalies;
        }
        
        let k = 0.5 * std;  // Allowable slack
        let h = 5.0 * std;  // Decision interval
        
        let mut cusum_pos = 0.0;
        let mut cusum_neg = 0.0;
        
        for (i, &x) in data.iter().enumerate() {
            cusum_pos = (cusum_pos + x - mean - k).max(0.0);
            cusum_neg = (cusum_neg - x + mean - k).max(0.0);
            
            if cusum_pos > h {
                anomalies.push(Anomaly {
                    index: i,
                    value: x,
                    score: cusum_pos / h,
                    anomaly_type: AnomalyType::ChangePoint,
                    confidence: (cusum_pos / h).min(1.0),
                });
                cusum_pos = 0.0;
            }
            
            if cusum_neg > h {
                anomalies.push(Anomaly {
                    index: i,
                    value: x,
                    score: cusum_neg / h,
                    anomaly_type: AnomalyType::ChangePoint,
                    confidence: (cusum_neg / h).min(1.0),
                });
                cusum_neg = 0.0;
            }
        }
        
        anomalies
    }
    
    /// Local Outlier Factor (simplified 1D version)
    fn detect_lof(&self, data: &[f64]) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        
        if data.len() < 20 {
            return anomalies;
        }
        
        let k = 5;  // Number of neighbors
        
        // For each point, calculate LOF
        for i in 0..data.len() {
            let x = data[i];
            
            // Find k nearest neighbors
            let mut distances: Vec<(usize, f64)> = data.iter()
                .enumerate()
                .filter(|&(j, _)| i != j)
                .map(|(j, &y)| (j, (x - y).abs()))
                .collect();
            distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            
            let k_neighbors: Vec<_> = distances.iter().take(k).collect();
            
            if k_neighbors.is_empty() {
                continue;
            }
            
            // k-distance (distance to k-th neighbor)
            let k_dist = k_neighbors.last().map(|(_, d)| *d).unwrap_or(0.0);
            
            // Local reachability density
            let lrd = if k_dist > 1e-10 {
                k as f64 / k_neighbors.iter().map(|(_, d)| d.max(k_dist)).sum::<f64>()
            } else {
                f64::MAX
            };
            
            // Calculate LOF (simplified)
            let mut neighbor_lrds = Vec::new();
            for &(j, _) in &k_neighbors {
                let neighbor_x = data[*j];
                let mut n_dists: Vec<f64> = data.iter()
                    .enumerate()
                    .filter(|&(idx, _)| idx != *j)
                    .map(|(_, &y)| (neighbor_x - y).abs())
                    .collect();
                n_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
                
                let n_k_dist = n_dists.get(k-1).copied().unwrap_or(0.0);
                if n_k_dist > 1e-10 {
                    let n_lrd = k as f64 / n_dists.iter().take(k).map(|d| d.max(n_k_dist)).sum::<f64>();
                    neighbor_lrds.push(n_lrd);
                }
            }
            
            if !neighbor_lrds.is_empty() && lrd < f64::MAX {
                let avg_neighbor_lrd = neighbor_lrds.iter().sum::<f64>() / neighbor_lrds.len() as f64;
                let lof = avg_neighbor_lrd / lrd;
                
                if lof > 1.5 {  // LOF threshold
                    anomalies.push(Anomaly {
                        index: i,
                        value: x,
                        score: lof,
                        anomaly_type: AnomalyType::ContextualAnomaly,
                        confidence: ((lof - 1.0) / 2.0).min(1.0),
                    });
                }
            }
        }
        
        anomalies
    }
    
    fn deduplicate_anomalies(&self, anomalies: &mut Vec<Anomaly>) {
        // Keep highest scoring anomaly for each index
        anomalies.sort_by_key(|a| a.index);
        let mut seen = std::collections::HashSet::new();
        anomalies.retain(|a| seen.insert(a.index));
    }
    
    fn expected_path_length(&self, n: usize) -> f64 {
        if n <= 1 {
            return 0.0;
        }
        let n = n as f64;
        2.0 * (n.ln() + 0.5772156649) - 2.0 * (n - 1.0) / n
    }
    
    fn z_score_to_confidence(&self, z: f64) -> f64 {
        // Approximate conversion using error function
        let x = z / 2.0_f64.sqrt();
        let t = 1.0 / (1.0 + 0.3275911 * x.abs());
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let erf = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
        let erf = if x >= 0.0 { erf } else { -erf };
        
        ((1.0 + erf) / 2.0 - 0.5).abs() * 2.0  // Two-tailed
    }
    
    fn std_dev(&self, data: &[f64]) -> f64 {
        if data.len() < 2 { return 0.0; }
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        variance.sqrt()
    }
    
    fn median(&self, data: &[f64]) -> f64 {
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }
    
    fn mad(&self, data: &[f64], median: f64) -> f64 {
        let deviations: Vec<f64> = data.iter().map(|&x| (x - median).abs()).collect();
        self.median(&deviations)
    }
}

/// Isolation Tree for Isolation Forest
struct IsolationTree {
    root: Option<Box<IsolationNode>>,
    max_depth: usize,
}

struct IsolationNode {
    split_value: f64,
    left: Option<Box<IsolationNode>>,
    right: Option<Box<IsolationNode>>,
    size: usize,
}

impl IsolationTree {
    fn build(data: &[f64], sample_size: usize) -> Self {
        use rand::prelude::*;
        
        let mut rng = thread_rng();
        let sample: Vec<f64> = data.choose_multiple(&mut rng, sample_size.min(data.len()))
            .cloned().collect();
        
        let max_depth = (sample_size as f64).log2().ceil() as usize;
        
        Self {
            root: Self::build_node(&sample, 0, max_depth, &mut rng),
            max_depth,
        }
    }
    
    fn build_node(data: &[f64], depth: usize, max_depth: usize, rng: &mut ThreadRng) -> Option<Box<IsolationNode>> {
        if data.is_empty() || depth >= max_depth {
            return None;
        }
        
        if data.len() == 1 {
            return Some(Box::new(IsolationNode {
                split_value: data[0],
                left: None,
                right: None,
                size: 1,
            }));
        }
        
        let min = data.iter().cloned().fold(f64::MAX, f64::min);
        let max = data.iter().cloned().fold(f64::MIN, f64::max);
        
        if (max - min).abs() < 1e-10 {
            return Some(Box::new(IsolationNode {
                split_value: min,
                left: None,
                right: None,
                size: data.len(),
            }));
        }
        
        let split_value = rng.gen_range(min..max);
        
        let left_data: Vec<f64> = data.iter().filter(|&&x| x < split_value).cloned().collect();
        let right_data: Vec<f64> = data.iter().filter(|&&x| x >= split_value).cloned().collect();
        
        Some(Box::new(IsolationNode {
            split_value,
            left: Self::build_node(&left_data, depth + 1, max_depth, rng),
            right: Self::build_node(&right_data, depth + 1, max_depth, rng),
            size: data.len(),
        }))
    }
    
    fn path_length(&self, value: f64) -> usize {
        self.path_length_recursive(&self.root, value, 0)
    }
    
    fn path_length_recursive(&self, node: &Option<Box<IsolationNode>>, value: f64, depth: usize) -> usize {
        match node {
            None => depth,
            Some(n) => {
                if n.left.is_none() && n.right.is_none() {
                    return depth + self.c(n.size);
                }
                
                if value < n.split_value {
                    self.path_length_recursive(&n.left, value, depth + 1)
                } else {
                    self.path_length_recursive(&n.right, value, depth + 1)
                }
            }
        }
    }
    
    fn c(&self, n: usize) -> usize {
        if n <= 1 { return 0; }
        let n = n as f64;
        (2.0 * (n.ln() + 0.5772156649) - 2.0 * (n - 1.0) / n) as usize
    }
}

use rand::prelude::*;
