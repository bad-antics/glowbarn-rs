//! Complexity measures - fractal dimension, recurrence plots, etc.

use serde::{Deserialize, Serialize};

/// Complexity analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityResult {
    pub fractal_dimension: f64,
    pub correlation_dimension: f64,
    pub lyapunov_exponent: f64,
    pub recurrence_rate: f64,
    pub determinism: f64,
    pub laminarity: f64,
    pub entropy_rate: f64,
}

/// Complexity analyzer
pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self
    }
    
    pub fn analyze(&self, data: &[f64]) -> ComplexityResult {
        if data.len() < 100 {
            return ComplexityResult::default();
        }
        
        let fractal_dimension = self.box_counting_dimension(data);
        let correlation_dimension = self.correlation_dimension(data);
        let lyapunov_exponent = self.estimate_lyapunov(data);
        
        let rqa = self.recurrence_quantification(data);
        
        ComplexityResult {
            fractal_dimension,
            correlation_dimension,
            lyapunov_exponent,
            recurrence_rate: rqa.0,
            determinism: rqa.1,
            laminarity: rqa.2,
            entropy_rate: self.entropy_rate(data),
        }
    }
    
    /// Box-counting fractal dimension
    fn box_counting_dimension(&self, data: &[f64]) -> f64 {
        let n = data.len();
        if n < 16 {
            return 1.0;
        }
        
        // Normalize data to [0, 1]
        let min = data.iter().cloned().fold(f64::MAX, f64::min);
        let max = data.iter().cloned().fold(f64::MIN, f64::max);
        let range = (max - min).max(1e-10);
        
        let normalized: Vec<f64> = data.iter()
            .map(|&x| (x - min) / range)
            .collect();
        
        // Count boxes at different scales
        let mut log_n = Vec::new();
        let mut log_r = Vec::new();
        
        for scale in [8, 16, 32, 64, 128].iter() {
            if *scale > n {
                break;
            }
            
            let box_size = 1.0 / *scale as f64;
            let mut boxes = std::collections::HashSet::new();
            
            for i in 0..n {
                let t = i as f64 / n as f64;
                let v = normalized[i];
                
                let box_t = (t / box_size) as i32;
                let box_v = (v / box_size) as i32;
                
                boxes.insert((box_t, box_v));
            }
            
            log_n.push((boxes.len() as f64).ln());
            log_r.push(box_size.ln());
        }
        
        // Linear regression for slope
        if log_n.len() < 2 {
            return 1.0;
        }
        
        let n_points = log_n.len() as f64;
        let sum_x: f64 = log_r.iter().sum();
        let sum_y: f64 = log_n.iter().sum();
        let sum_xy: f64 = log_r.iter().zip(log_n.iter()).map(|(x, y)| x * y).sum();
        let sum_xx: f64 = log_r.iter().map(|x| x * x).sum();
        
        let slope = (n_points * sum_xy - sum_x * sum_y) / (n_points * sum_xx - sum_x * sum_x);
        
        (-slope).clamp(1.0, 2.0)
    }
    
    /// Correlation dimension (Grassberger-Procaccia algorithm)
    fn correlation_dimension(&self, data: &[f64]) -> f64 {
        let n = data.len();
        if n < 50 {
            return 1.0;
        }
        
        let embedding_dim = 3;
        let delay = 1;
        
        // Create embedded vectors
        let n_vectors = n - (embedding_dim - 1) * delay;
        if n_vectors < 20 {
            return 1.0;
        }
        
        let vectors: Vec<Vec<f64>> = (0..n_vectors)
            .map(|i| {
                (0..embedding_dim)
                    .map(|d| data[i + d * delay])
                    .collect()
            })
            .collect();
        
        // Compute correlation integral at different scales
        let mut log_c = Vec::new();
        let mut log_r = Vec::new();
        
        // Estimate scale range
        let mut all_dists = Vec::new();
        for i in 0..n_vectors.min(100) {
            for j in (i+1)..n_vectors.min(100) {
                let dist: f64 = vectors[i].iter()
                    .zip(vectors[j].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>().sqrt();
                all_dists.push(dist);
            }
        }
        all_dists.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let r_min = all_dists.get(all_dists.len() / 10).copied().unwrap_or(0.01);
        let r_max = all_dists.get(all_dists.len() * 9 / 10).copied().unwrap_or(1.0);
        
        for i in 0..10 {
            let r = r_min * (r_max / r_min).powf(i as f64 / 9.0);
            
            let mut count = 0;
            for i in 0..n_vectors {
                for j in (i+1)..n_vectors {
                    let dist: f64 = vectors[i].iter()
                        .zip(vectors[j].iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>().sqrt();
                    if dist < r {
                        count += 1;
                    }
                }
            }
            
            let c = 2.0 * count as f64 / (n_vectors * (n_vectors - 1)) as f64;
            if c > 0.0 {
                log_c.push(c.ln());
                log_r.push(r.ln());
            }
        }
        
        // Linear regression
        if log_c.len() < 3 {
            return 1.0;
        }
        
        let n_points = log_c.len() as f64;
        let sum_x: f64 = log_r.iter().sum();
        let sum_y: f64 = log_c.iter().sum();
        let sum_xy: f64 = log_r.iter().zip(log_c.iter()).map(|(x, y)| x * y).sum();
        let sum_xx: f64 = log_r.iter().map(|x| x * x).sum();
        
        let slope = (n_points * sum_xy - sum_x * sum_y) / (n_points * sum_xx - sum_x * sum_x);
        
        slope.clamp(0.1, 10.0)
    }
    
    /// Estimate largest Lyapunov exponent
    fn estimate_lyapunov(&self, data: &[f64]) -> f64 {
        let n = data.len();
        if n < 100 {
            return 0.0;
        }
        
        let embedding_dim = 3;
        let delay = 1;
        let n_vectors = n - (embedding_dim - 1) * delay;
        
        if n_vectors < 50 {
            return 0.0;
        }
        
        // Create embedded vectors
        let vectors: Vec<Vec<f64>> = (0..n_vectors)
            .map(|i| {
                (0..embedding_dim)
                    .map(|d| data[i + d * delay])
                    .collect()
            })
            .collect();
        
        // Find nearest neighbors and track divergence
        let mut sum_log_div = 0.0;
        let mut count = 0;
        
        for i in 0..(n_vectors - 10) {
            // Find nearest neighbor (not too close in time)
            let mut min_dist = f64::MAX;
            let mut nn_idx = 0;
            
            for j in 0..n_vectors {
                if (i as i32 - j as i32).abs() < 10 {
                    continue;
                }
                
                let dist: f64 = vectors[i].iter()
                    .zip(vectors[j].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>().sqrt();
                
                if dist < min_dist && dist > 1e-10 {
                    min_dist = dist;
                    nn_idx = j;
                }
            }
            
            if min_dist < f64::MAX && nn_idx + 10 < n_vectors && i + 10 < n_vectors {
                // Track divergence over 10 steps
                let final_dist: f64 = vectors[i + 10].iter()
                    .zip(vectors[nn_idx + 10].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>().sqrt();
                
                if final_dist > 1e-10 && min_dist > 1e-10 {
                    sum_log_div += (final_dist / min_dist).ln();
                    count += 1;
                }
            }
        }
        
        if count > 0 {
            sum_log_div / (count as f64 * 10.0)  // Per time step
        } else {
            0.0
        }
    }
    
    /// Recurrence quantification analysis
    fn recurrence_quantification(&self, data: &[f64]) -> (f64, f64, f64) {
        let n = data.len();
        if n < 50 {
            return (0.0, 0.0, 0.0);
        }
        
        let embedding_dim = 2;
        let delay = 1;
        let n_vectors = n - (embedding_dim - 1) * delay;
        
        // Create embedded vectors
        let vectors: Vec<Vec<f64>> = (0..n_vectors)
            .map(|i| {
                (0..embedding_dim)
                    .map(|d| data[i + d * delay])
                    .collect()
            })
            .collect();
        
        // Compute threshold (10% of max distance)
        let mut max_dist: f64 = 0.0;
        for i in 0..n_vectors.min(50) {
            for j in (i+1)..n_vectors.min(50) {
                let dist: f64 = vectors[i].iter()
                    .zip(vectors[j].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>().sqrt();
                max_dist = max_dist.max(dist);
            }
        }
        let threshold = 0.1 * max_dist;
        
        // Build recurrence matrix (sparse)
        let mut recurrence_points = 0;
        let mut diagonal_lines = Vec::new();
        let mut vertical_lines: Vec<usize> = Vec::new();
        
        // Count recurrence rate and diagonal lines
        for i in 0..n_vectors {
            let mut diag_length = 0;
            let mut vert_length = 0;
            
            for j in 0..n_vectors {
                let dist: f64 = vectors[i].iter()
                    .zip(vectors[j].iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f64>().sqrt();
                
                let is_recurrent = dist < threshold;
                
                if is_recurrent {
                    recurrence_points += 1;
                    
                    // Diagonal line check
                    if i > 0 && j > 0 {
                        let prev_dist: f64 = vectors[i-1].iter()
                            .zip(vectors[j-1].iter())
                            .map(|(a, b)| (a - b).powi(2))
                            .sum::<f64>().sqrt();
                        if prev_dist < threshold {
                            diag_length += 1;
                        }
                    }
                }
            }
            
            if diag_length > 1 {
                diagonal_lines.push(diag_length);
            }
        }
        
        let total_points = (n_vectors * n_vectors) as f64;
        let recurrence_rate = recurrence_points as f64 / total_points;
        
        // Determinism: fraction of recurrence points in diagonal lines
        let diag_points: usize = diagonal_lines.iter().sum();
        let determinism = if recurrence_points > 0 {
            diag_points as f64 / recurrence_points as f64
        } else {
            0.0
        };
        
        // Laminarity: fraction in vertical lines (simplified)
        let laminarity = determinism * 0.8;  // Approximation
        
        (recurrence_rate, determinism, laminarity)
    }
    
    /// Entropy rate estimation
    fn entropy_rate(&self, data: &[f64]) -> f64 {
        if data.len() < 100 {
            return 0.0;
        }
        
        // Block entropy approach
        let block_sizes = [1, 2, 4, 8];
        let mut entropies = Vec::new();
        
        // Discretize data
        let min = data.iter().cloned().fold(f64::MAX, f64::min);
        let max = data.iter().cloned().fold(f64::MIN, f64::max);
        let range = (max - min).max(1e-10);
        let n_symbols = 16;
        
        let symbols: Vec<usize> = data.iter()
            .map(|&x| (((x - min) / range) * (n_symbols - 1) as f64) as usize)
            .collect();
        
        for &block_size in &block_sizes {
            if symbols.len() < block_size * 10 {
                continue;
            }
            
            let mut counts = std::collections::HashMap::new();
            for chunk in symbols.windows(block_size) {
                *counts.entry(chunk.to_vec()).or_insert(0usize) += 1;
            }
            
            let total = (symbols.len() - block_size + 1) as f64;
            let entropy: f64 = counts.values()
                .map(|&c| {
                    let p = c as f64 / total;
                    if p > 0.0 { -p * p.log2() } else { 0.0 }
                })
                .sum();
            
            entropies.push((block_size as f64, entropy));
        }
        
        // Entropy rate is slope of H(n) vs n
        if entropies.len() < 2 {
            return 0.0;
        }
        
        let n = entropies.len() as f64;
        let sum_x: f64 = entropies.iter().map(|(x, _)| x).sum();
        let sum_y: f64 = entropies.iter().map(|(_, y)| y).sum();
        let sum_xy: f64 = entropies.iter().map(|(x, y)| x * y).sum();
        let sum_xx: f64 = entropies.iter().map(|(x, _)| x * x).sum();
        
        (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x)
    }
}
