// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Entropy analysis - Shannon, Sample, Spectral, Permutation, Approximate, Kolmogorov

use std::collections::HashMap;
use std::f64::consts::{E, PI};
use rustfft::{FftPlanner, num_complex::Complex};
use serde::{Deserialize, Serialize};

use super::AnalysisConfig;

/// Result of entropy analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyResult {
    // Classic entropy measures
    pub shannon: f64,
    pub renyi: f64,        // Rényi entropy (alpha=2)
    pub tsallis: f64,      // Tsallis entropy (q=2)
    
    // Time-series entropy
    pub sample: f64,
    pub approximate: f64,
    pub permutation: f64,
    pub multiscale: Vec<f64>,
    
    // Spectral entropy
    pub spectral: f64,
    pub wavelet: f64,
    
    // Complexity measures
    pub lz_complexity: f64,
    pub kolmogorov_estimate: f64,
    pub hurst_exponent: f64,
    
    // Statistical
    pub kurtosis: f64,
    pub skewness: f64,
    
    // Anomaly flag
    pub is_anomalous: bool,
    pub anomaly_score: f64,
}

/// Entropy analyzer
pub struct EntropyAnalyzer {
    config: AnalysisConfig,
    fft_planner: FftPlanner<f64>,
    baseline_entropy: Option<f64>,
}

impl EntropyAnalyzer {
    pub fn new(config: AnalysisConfig) -> Self {
        Self {
            config,
            fft_planner: FftPlanner::new(),
            baseline_entropy: None,
        }
    }
    
    pub fn analyze(&self, data: &[f64]) -> EntropyResult {
        if data.is_empty() {
            return EntropyResult::default();
        }
        
        // Compute all entropy measures
        let shannon = self.shannon_entropy(data);
        let renyi = self.renyi_entropy(data, 2.0);
        let tsallis = self.tsallis_entropy(data, 2.0);
        
        let sample = self.sample_entropy(data, 2, 0.2);
        let approximate = self.approximate_entropy(data, 2, 0.2);
        let permutation = self.permutation_entropy(data, 3, 1);
        let multiscale = self.multiscale_entropy(data, 2, 0.2, 10);
        
        let spectral = self.spectral_entropy(data);
        let wavelet = self.wavelet_entropy(data);
        
        let lz_complexity = self.lempel_ziv_complexity(data);
        let kolmogorov_estimate = self.estimate_kolmogorov(data);
        let hurst = self.hurst_exponent(data);
        
        let (skewness, kurtosis) = self.compute_moments(data);
        
        // Anomaly detection based on entropy deviation
        let anomaly_score = self.compute_anomaly_score(shannon, sample, spectral);
        let is_anomalous = anomaly_score > self.config.anomaly_threshold;
        
        EntropyResult {
            shannon, renyi, tsallis,
            sample, approximate, permutation, multiscale,
            spectral, wavelet,
            lz_complexity, kolmogorov_estimate, hurst_exponent: hurst,
            kurtosis, skewness,
            is_anomalous, anomaly_score,
        }
    }
    
    /// Shannon entropy: H = -Σ p(x) log2(p(x))
    pub fn shannon_entropy(&self, data: &[f64]) -> f64 {
        let mut histogram = HashMap::new();
        let bins = 256;
        
        let (min, max) = data.iter().fold((f64::MAX, f64::MIN), |(min, max), &x| {
            (min.min(x), max.max(x))
        });
        
        let range = (max - min).max(1e-10);
        
        for &x in data {
            let bin = (((x - min) / range) * (bins - 1) as f64) as usize;
            *histogram.entry(bin).or_insert(0usize) += 1;
        }
        
        let n = data.len() as f64;
        histogram.values()
            .map(|&count| {
                let p = count as f64 / n;
                if p > 0.0 { -p * p.log2() } else { 0.0 }
            })
            .sum()
    }
    
    /// Rényi entropy: H_α = (1/(1-α)) * log(Σ p(x)^α)
    pub fn renyi_entropy(&self, data: &[f64], alpha: f64) -> f64 {
        if (alpha - 1.0).abs() < 1e-10 {
            return self.shannon_entropy(data);
        }
        
        let mut histogram = HashMap::new();
        let bins = 256;
        
        let (min, max) = data.iter().fold((f64::MAX, f64::MIN), |(min, max), &x| {
            (min.min(x), max.max(x))
        });
        let range = (max - min).max(1e-10);
        
        for &x in data {
            let bin = (((x - min) / range) * (bins - 1) as f64) as usize;
            *histogram.entry(bin).or_insert(0usize) += 1;
        }
        
        let n = data.len() as f64;
        let sum_p_alpha: f64 = histogram.values()
            .map(|&count| (count as f64 / n).powf(alpha))
            .sum();
        
        (1.0 / (1.0 - alpha)) * sum_p_alpha.log2()
    }
    
    /// Tsallis entropy: S_q = (1/(q-1)) * (1 - Σ p(x)^q)
    pub fn tsallis_entropy(&self, data: &[f64], q: f64) -> f64 {
        if (q - 1.0).abs() < 1e-10 {
            return self.shannon_entropy(data);
        }
        
        let mut histogram = HashMap::new();
        let bins = 256;
        
        let (min, max) = data.iter().fold((f64::MAX, f64::MIN), |(min, max), &x| {
            (min.min(x), max.max(x))
        });
        let range = (max - min).max(1e-10);
        
        for &x in data {
            let bin = (((x - min) / range) * (bins - 1) as f64) as usize;
            *histogram.entry(bin).or_insert(0usize) += 1;
        }
        
        let n = data.len() as f64;
        let sum_p_q: f64 = histogram.values()
            .map(|&count| (count as f64 / n).powf(q))
            .sum();
        
        (1.0 - sum_p_q) / (q - 1.0)
    }
    
    /// Sample entropy - measures regularity
    pub fn sample_entropy(&self, data: &[f64], m: usize, r_mult: f64) -> f64 {
        let n = data.len();
        if n < m + 2 {
            return 0.0;
        }
        
        let std_dev = self.std_dev(data);
        let r = r_mult * std_dev;
        
        let mut count_m = 0usize;
        let mut count_m1 = 0usize;
        
        // Count template matches for embedding dimension m
        for i in 0..(n - m) {
            for j in (i + 1)..(n - m) {
                let mut match_m = true;
                let mut match_m1 = true;
                
                // Check m-length match
                for k in 0..m {
                    if (data[i + k] - data[j + k]).abs() > r {
                        match_m = false;
                        match_m1 = false;
                        break;
                    }
                }
                
                if match_m {
                    count_m += 1;
                    
                    // Check (m+1)-length match
                    if i + m < n && j + m < n {
                        if (data[i + m] - data[j + m]).abs() <= r {
                            count_m1 += 1;
                        }
                    }
                }
            }
        }
        
        if count_m == 0 || count_m1 == 0 {
            return 0.0;
        }
        
        -((count_m1 as f64) / (count_m as f64)).ln()
    }
    
    /// Approximate entropy - similar to sample entropy but includes self-matches
    pub fn approximate_entropy(&self, data: &[f64], m: usize, r_mult: f64) -> f64 {
        let phi_m = self.phi(data, m, r_mult);
        let phi_m1 = self.phi(data, m + 1, r_mult);
        phi_m - phi_m1
    }
    
    fn phi(&self, data: &[f64], m: usize, r_mult: f64) -> f64 {
        let n = data.len();
        if n < m {
            return 0.0;
        }
        
        let std_dev = self.std_dev(data);
        let r = r_mult * std_dev;
        
        let mut c = vec![0.0; n - m + 1];
        
        for i in 0..(n - m + 1) {
            let mut count = 0;
            for j in 0..(n - m + 1) {
                let mut within_r = true;
                for k in 0..m {
                    if (data[i + k] - data[j + k]).abs() > r {
                        within_r = false;
                        break;
                    }
                }
                if within_r {
                    count += 1;
                }
            }
            c[i] = count as f64 / (n - m + 1) as f64;
        }
        
        c.iter().map(|&x| if x > 0.0 { x.ln() } else { 0.0 }).sum::<f64>() / (n - m + 1) as f64
    }
    
    /// Permutation entropy
    pub fn permutation_entropy(&self, data: &[f64], m: usize, delay: usize) -> f64 {
        if data.len() < m * delay {
            return 0.0;
        }
        
        let mut patterns: HashMap<Vec<usize>, usize> = HashMap::new();
        let n_patterns = data.len() - (m - 1) * delay;
        
        for i in 0..n_patterns {
            let mut indices: Vec<usize> = (0..m).collect();
            indices.sort_by(|&a, &b| {
                let va = data[i + a * delay];
                let vb = data[i + b * delay];
                va.partial_cmp(&vb).unwrap()
            });
            
            *patterns.entry(indices).or_insert(0) += 1;
        }
        
        let n = n_patterns as f64;
        let max_entropy = (1..=m).map(|i| i as f64).product::<f64>().ln();
        
        let entropy: f64 = patterns.values()
            .map(|&count| {
                let p = count as f64 / n;
                if p > 0.0 { -p * p.ln() } else { 0.0 }
            })
            .sum();
        
        entropy / max_entropy  // Normalized
    }
    
    /// Multiscale entropy
    pub fn multiscale_entropy(&self, data: &[f64], m: usize, r: f64, scales: usize) -> Vec<f64> {
        (1..=scales).map(|scale| {
            let coarse = self.coarse_grain(data, scale);
            self.sample_entropy(&coarse, m, r)
        }).collect()
    }
    
    fn coarse_grain(&self, data: &[f64], scale: usize) -> Vec<f64> {
        data.chunks(scale)
            .map(|chunk| chunk.iter().sum::<f64>() / chunk.len() as f64)
            .collect()
    }
    
    /// Spectral entropy
    pub fn spectral_entropy(&self, data: &[f64]) -> f64 {
        if data.len() < 4 {
            return 0.0;
        }
        
        let n = data.len().next_power_of_two();
        let mut buffer: Vec<Complex<f64>> = data.iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();
        buffer.resize(n, Complex::new(0.0, 0.0));
        
        // Create a new planner for this call
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);
        fft.process(&mut buffer);
        
        // Power spectrum (only positive frequencies)
        let power: Vec<f64> = buffer[0..n/2].iter()
            .map(|c| c.norm_sqr())
            .collect();
        
        let total: f64 = power.iter().sum();
        if total < 1e-10 {
            return 0.0;
        }
        
        // Normalized power spectral density
        let psd: Vec<f64> = power.iter().map(|&p| p / total).collect();
        
        // Shannon entropy of PSD
        let max_entropy = (n as f64 / 2.0).log2();
        let entropy: f64 = psd.iter()
            .map(|&p| if p > 0.0 { -p * p.log2() } else { 0.0 })
            .sum();
        
        entropy / max_entropy  // Normalized
    }
    
    /// Wavelet entropy (simplified Haar wavelet)
    pub fn wavelet_entropy(&self, data: &[f64]) -> f64 {
        if data.len() < 8 {
            return 0.0;
        }
        
        // Simple Haar wavelet decomposition
        let mut levels = Vec::new();
        let mut current: Vec<f64> = data.to_vec();
        
        while current.len() >= 2 {
            let mut approx = Vec::new();
            let mut detail = Vec::new();
            
            for chunk in current.chunks_exact(2) {
                approx.push((chunk[0] + chunk[1]) / 2.0_f64.sqrt());
                detail.push((chunk[0] - chunk[1]) / 2.0_f64.sqrt());
            }
            
            let energy: f64 = detail.iter().map(|x| x * x).sum();
            levels.push(energy);
            current = approx;
        }
        
        let total: f64 = levels.iter().sum();
        if total < 1e-10 {
            return 0.0;
        }
        
        // Entropy of energy distribution across scales
        let n = levels.len() as f64;
        levels.iter()
            .map(|&e| {
                let p = e / total;
                if p > 0.0 { -p * p.log2() } else { 0.0 }
            })
            .sum::<f64>() / n.log2()
    }
    
    /// Lempel-Ziv complexity
    pub fn lempel_ziv_complexity(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        
        // Binarize data
        let median = self.median(data);
        let binary: Vec<u8> = data.iter()
            .map(|&x| if x >= median { 1 } else { 0 })
            .collect();
        
        let n = binary.len();
        let mut i = 0;
        let mut c = 1;  // Complexity count
        let mut k = 1;
        let mut k_max = 1;
        
        while i + k <= n {
            let substring = &binary[i..i+k];
            let search_end = i + k - 1;
            
            // Check if substring exists in prefix
            let exists = (0..search_end).any(|j| {
                if j + k > search_end { return false; }
                &binary[j..j+k] == substring
            });
            
            if exists {
                k += 1;
            } else {
                c += 1;
                i += k;
                k_max = k_max.max(k);
                k = 1;
            }
        }
        
        // Normalize by theoretical maximum
        let h = (n as f64) / (n as f64).log2();
        (c as f64) / h
    }
    
    /// Estimate Kolmogorov complexity (via compression)
    pub fn estimate_kolmogorov(&self, data: &[f64]) -> f64 {
        // Simple estimation using LZ complexity
        // Real Kolmogorov complexity is uncomputable, this is an approximation
        let lz = self.lempel_ziv_complexity(data);
        let n = data.len() as f64;
        
        // Normalized complexity estimate
        lz * n.log2() / n
    }
    
    /// Hurst exponent via R/S analysis
    pub fn hurst_exponent(&self, data: &[f64]) -> f64 {
        if data.len() < 32 {
            return 0.5;
        }
        
        let n = data.len();
        let mut rs_values = Vec::new();
        let mut n_values = Vec::new();
        
        // Calculate R/S for different subseries lengths
        for size in (8..n/2).step_by(4) {
            let mut rs_sum = 0.0;
            let mut count = 0;
            
            for i in (0..n).step_by(size) {
                if i + size > n { break; }
                
                let subset = &data[i..i+size];
                let mean = subset.iter().sum::<f64>() / size as f64;
                
                // Cumulative deviation from mean
                let mut cumsum = Vec::with_capacity(size);
                let mut sum = 0.0;
                for &x in subset {
                    sum += x - mean;
                    cumsum.push(sum);
                }
                
                let r = cumsum.iter().cloned().fold(f64::MIN, f64::max) 
                      - cumsum.iter().cloned().fold(f64::MAX, f64::min);
                let s = self.std_dev(subset);
                
                if s > 1e-10 {
                    rs_sum += r / s;
                    count += 1;
                }
            }
            
            if count > 0 {
                rs_values.push((rs_sum / count as f64).ln());
                n_values.push((size as f64).ln());
            }
        }
        
        // Linear regression to find Hurst exponent
        if rs_values.len() < 2 {
            return 0.5;
        }
        
        let n = rs_values.len() as f64;
        let sum_x: f64 = n_values.iter().sum();
        let sum_y: f64 = rs_values.iter().sum();
        let sum_xy: f64 = n_values.iter().zip(rs_values.iter()).map(|(x, y)| x * y).sum();
        let sum_xx: f64 = n_values.iter().map(|x| x * x).sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        slope.clamp(0.0, 1.0)
    }
    
    fn compute_moments(&self, data: &[f64]) -> (f64, f64) {
        if data.len() < 4 {
            return (0.0, 0.0);
        }
        
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let std = self.std_dev(data);
        
        if std < 1e-10 {
            return (0.0, 0.0);
        }
        
        let skewness = data.iter()
            .map(|&x| ((x - mean) / std).powi(3))
            .sum::<f64>() / n;
        
        let kurtosis = data.iter()
            .map(|&x| ((x - mean) / std).powi(4))
            .sum::<f64>() / n - 3.0;  // Excess kurtosis
        
        (skewness, kurtosis)
    }
    
    fn compute_anomaly_score(&self, shannon: f64, sample: f64, spectral: f64) -> f64 {
        // Combine entropy measures for anomaly detection
        // High entropy + low sample entropy = potentially anomalous
        
        let baseline = self.baseline_entropy.unwrap_or(shannon);
        let shannon_dev = (shannon - baseline).abs() / baseline.max(1e-10);
        
        // Sample entropy close to 0 indicates regularity (potentially artificial)
        let regularity_score = if sample < 0.1 { 2.0 } else { 0.0 };
        
        // Very high or very low spectral entropy
        let spectral_score = if spectral < 0.2 || spectral > 0.95 { 1.5 } else { 0.0 };
        
        shannon_dev + regularity_score + spectral_score
    }
    
    fn std_dev(&self, data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        variance.sqrt()
    }
    
    fn median(&self, data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let mut sorted = data.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    }
}

impl Default for EntropyResult {
    fn default() -> Self {
        Self {
            shannon: 0.0,
            renyi: 0.0,
            tsallis: 0.0,
            sample: 0.0,
            approximate: 0.0,
            permutation: 0.0,
            multiscale: vec![],
            spectral: 0.0,
            wavelet: 0.0,
            lz_complexity: 0.0,
            kolmogorov_estimate: 0.0,
            hurst_exponent: 0.5,
            kurtosis: 0.0,
            skewness: 0.0,
            is_anomalous: false,
            anomaly_score: 0.0,
        }
    }
}
