// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Signal processing - FFT, filtering, feature extraction

use std::f64::consts::PI;
use rustfft::{FftPlanner, num_complex::Complex};
use serde::{Deserialize, Serialize};

use super::AnalysisConfig;

/// Signal features extracted from waveform
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SignalFeatures {
    // Time domain
    pub mean: f64,
    pub std_dev: f64,
    pub rms: f64,
    pub peak_to_peak: f64,
    pub crest_factor: f64,
    pub zero_crossings: usize,
    
    // Frequency domain
    pub dominant_frequency: f64,
    pub spectral_centroid: f64,
    pub spectral_bandwidth: f64,
    pub spectral_rolloff: f64,
    pub spectral_flatness: f64,
    
    // Frequency bands (for audio)
    pub band_energies: Vec<f64>,
    
    // Temporal
    pub attack_time: f64,
    pub decay_time: f64,
}

/// Signal processor for waveform analysis
pub struct SignalProcessor {
    config: AnalysisConfig,
}

impl SignalProcessor {
    pub fn new(config: AnalysisConfig) -> Self {
        Self { config }
    }
    
    pub fn extract_features(&self, data: &[f64], sample_rate: f64) -> SignalFeatures {
        if data.is_empty() {
            return SignalFeatures::default();
        }
        
        let time_features = self.time_domain_features(data);
        let freq_features = self.frequency_domain_features(data, sample_rate);
        let temporal = self.temporal_features(data, sample_rate);
        
        SignalFeatures {
            mean: time_features.0,
            std_dev: time_features.1,
            rms: time_features.2,
            peak_to_peak: time_features.3,
            crest_factor: time_features.4,
            zero_crossings: time_features.5,
            dominant_frequency: freq_features.0,
            spectral_centroid: freq_features.1,
            spectral_bandwidth: freq_features.2,
            spectral_rolloff: freq_features.3,
            spectral_flatness: freq_features.4,
            band_energies: freq_features.5,
            attack_time: temporal.0,
            decay_time: temporal.1,
        }
    }
    
    fn time_domain_features(&self, data: &[f64]) -> (f64, f64, f64, f64, f64, usize) {
        let n = data.len() as f64;
        
        // Mean
        let mean = data.iter().sum::<f64>() / n;
        
        // Standard deviation
        let variance = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let std_dev = variance.sqrt();
        
        // RMS
        let rms = (data.iter().map(|&x| x * x).sum::<f64>() / n).sqrt();
        
        // Peak to peak
        let max = data.iter().cloned().fold(f64::MIN, f64::max);
        let min = data.iter().cloned().fold(f64::MAX, f64::min);
        let peak_to_peak = max - min;
        
        // Crest factor
        let crest_factor = if rms > 1e-10 { max.abs().max(min.abs()) / rms } else { 0.0 };
        
        // Zero crossings
        let zero_crossings = data.windows(2)
            .filter(|w| (w[0] - mean) * (w[1] - mean) < 0.0)
            .count();
        
        (mean, std_dev, rms, peak_to_peak, crest_factor, zero_crossings)
    }
    
    fn frequency_domain_features(&self, data: &[f64], sample_rate: f64) -> (f64, f64, f64, f64, f64, Vec<f64>) {
        if data.len() < 4 {
            return (0.0, 0.0, 0.0, 0.0, 0.0, vec![]);
        }
        
        let n = data.len().next_power_of_two();
        
        // Window the data (Hann window)
        let windowed: Vec<f64> = data.iter().enumerate()
            .map(|(i, &x)| x * 0.5 * (1.0 - (2.0 * PI * i as f64 / (data.len() - 1) as f64).cos()))
            .collect();
        
        // FFT
        let mut buffer: Vec<Complex<f64>> = windowed.iter()
            .map(|&x| Complex::new(x, 0.0))
            .collect();
        buffer.resize(n, Complex::new(0.0, 0.0));
        
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n);
        fft.process(&mut buffer);
        
        // Power spectrum (positive frequencies only)
        let power: Vec<f64> = buffer[0..n/2].iter()
            .map(|c| c.norm_sqr())
            .collect();
        
        let total_power: f64 = power.iter().sum();
        if total_power < 1e-10 {
            return (0.0, 0.0, 0.0, 0.0, 0.0, vec![]);
        }
        
        // Frequency resolution
        let freq_resolution = sample_rate / n as f64;
        
        // Dominant frequency
        let (max_idx, _) = power.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap();
        let dominant_frequency = max_idx as f64 * freq_resolution;
        
        // Spectral centroid
        let spectral_centroid = power.iter().enumerate()
            .map(|(i, &p)| i as f64 * freq_resolution * p)
            .sum::<f64>() / total_power;
        
        // Spectral bandwidth
        let spectral_bandwidth = (power.iter().enumerate()
            .map(|(i, &p)| (i as f64 * freq_resolution - spectral_centroid).powi(2) * p)
            .sum::<f64>() / total_power).sqrt();
        
        // Spectral rolloff (85% of energy)
        let target = total_power * 0.85;
        let mut cumsum = 0.0;
        let mut rolloff_idx = 0;
        for (i, &p) in power.iter().enumerate() {
            cumsum += p;
            if cumsum >= target {
                rolloff_idx = i;
                break;
            }
        }
        let spectral_rolloff = rolloff_idx as f64 * freq_resolution;
        
        // Spectral flatness (geometric mean / arithmetic mean)
        let arithmetic_mean = total_power / power.len() as f64;
        let log_sum: f64 = power.iter()
            .map(|&p| if p > 1e-10 { p.ln() } else { -23.0 })  // -23 ≈ ln(1e-10)
            .sum();
        let geometric_mean = (log_sum / power.len() as f64).exp();
        let spectral_flatness = if arithmetic_mean > 1e-10 {
            geometric_mean / arithmetic_mean
        } else {
            0.0
        };
        
        // Band energies (octave bands)
        let band_edges = [20.0, 40.0, 80.0, 160.0, 320.0, 640.0, 1280.0, 2560.0, 5120.0, 10240.0, 20480.0];
        let mut band_energies = Vec::new();
        
        for window in band_edges.windows(2) {
            let low = window[0];
            let high = window[1];
            let low_bin = (low / freq_resolution) as usize;
            let high_bin = ((high / freq_resolution) as usize).min(power.len());
            
            if low_bin < high_bin && low_bin < power.len() {
                let energy: f64 = power[low_bin..high_bin].iter().sum();
                band_energies.push(energy / total_power);
            }
        }
        
        (dominant_frequency, spectral_centroid, spectral_bandwidth, spectral_rolloff, spectral_flatness, band_energies)
    }
    
    fn temporal_features(&self, data: &[f64], sample_rate: f64) -> (f64, f64) {
        if data.len() < 10 {
            return (0.0, 0.0);
        }
        
        // Envelope detection (simple moving RMS)
        let window_size = (sample_rate * 0.01) as usize; // 10ms window
        let window_size = window_size.max(3).min(data.len() / 4);
        
        let envelope: Vec<f64> = data.windows(window_size)
            .map(|w| (w.iter().map(|&x| x * x).sum::<f64>() / w.len() as f64).sqrt())
            .collect();
        
        if envelope.is_empty() {
            return (0.0, 0.0);
        }
        
        let max_env = envelope.iter().cloned().fold(f64::MIN, f64::max);
        if max_env < 1e-10 {
            return (0.0, 0.0);
        }
        
        // Attack time (time to reach 90% of max)
        let attack_threshold = 0.9 * max_env;
        let attack_idx = envelope.iter()
            .position(|&e| e >= attack_threshold)
            .unwrap_or(envelope.len());
        let attack_time = attack_idx as f64 / sample_rate;
        
        // Decay time (time from peak to 10% of max)
        let (peak_idx, _) = envelope.iter().enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap_or((0, &0.0));
        
        let decay_threshold = 0.1 * max_env;
        let decay_idx = envelope[peak_idx..].iter()
            .position(|&e| e <= decay_threshold)
            .unwrap_or(envelope.len() - peak_idx);
        let decay_time = decay_idx as f64 / sample_rate;
        
        (attack_time, decay_time)
    }
    
    /// Apply bandpass filter
    pub fn bandpass_filter(&self, data: &[f64], sample_rate: f64, low_freq: f64, high_freq: f64) -> Vec<f64> {
        if data.len() < 8 {
            return data.to_vec();
        }
        
        // Simple IIR Butterworth bandpass (2nd order)
        let w0_low = 2.0 * PI * low_freq / sample_rate;
        let w0_high = 2.0 * PI * high_freq / sample_rate;
        
        let alpha_low = w0_low.sin() / (2.0 * 0.707);
        let alpha_high = w0_high.sin() / (2.0 * 0.707);
        
        // High-pass coefficients
        let hp_b0 = (1.0 + w0_low.cos()) / 2.0;
        let hp_b1 = -(1.0 + w0_low.cos());
        let hp_b2 = (1.0 + w0_low.cos()) / 2.0;
        let hp_a0 = 1.0 + alpha_low;
        let hp_a1 = -2.0 * w0_low.cos();
        let hp_a2 = 1.0 - alpha_low;
        
        // Low-pass coefficients
        let lp_b0 = (1.0 - w0_high.cos()) / 2.0;
        let lp_b1 = 1.0 - w0_high.cos();
        let lp_b2 = (1.0 - w0_high.cos()) / 2.0;
        let lp_a0 = 1.0 + alpha_high;
        let lp_a1 = -2.0 * w0_high.cos();
        let lp_a2 = 1.0 - alpha_high;
        
        // Apply high-pass
        let mut hp_out = vec![0.0; data.len()];
        for i in 2..data.len() {
            hp_out[i] = (hp_b0 / hp_a0) * data[i] 
                      + (hp_b1 / hp_a0) * data[i-1]
                      + (hp_b2 / hp_a0) * data[i-2]
                      - (hp_a1 / hp_a0) * hp_out[i-1]
                      - (hp_a2 / hp_a0) * hp_out[i-2];
        }
        
        // Apply low-pass
        let mut output = vec![0.0; data.len()];
        for i in 2..data.len() {
            output[i] = (lp_b0 / lp_a0) * hp_out[i]
                      + (lp_b1 / lp_a0) * hp_out[i-1]
                      + (lp_b2 / lp_a0) * hp_out[i-2]
                      - (lp_a1 / lp_a0) * output[i-1]
                      - (lp_a2 / lp_a0) * output[i-2];
        }
        
        output
    }
    
    /// Compute spectrogram
    pub fn spectrogram(&self, data: &[f64], sample_rate: f64, window_size: usize, hop_size: usize) -> Vec<Vec<f64>> {
        let mut spectrogram = Vec::new();
        let n_fft = window_size.next_power_of_two();
        
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(n_fft);
        
        let hann: Vec<f64> = (0..window_size)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f64 / (window_size - 1) as f64).cos()))
            .collect();
        
        let mut pos = 0;
        while pos + window_size <= data.len() {
            let mut buffer: Vec<Complex<f64>> = data[pos..pos+window_size].iter()
                .zip(hann.iter())
                .map(|(&x, &w)| Complex::new(x * w, 0.0))
                .collect();
            buffer.resize(n_fft, Complex::new(0.0, 0.0));
            
            fft.process(&mut buffer);
            
            let power: Vec<f64> = buffer[0..n_fft/2].iter()
                .map(|c| (c.norm_sqr() + 1e-10).log10() * 10.0)  // dB
                .collect();
            
            spectrogram.push(power);
            pos += hop_size;
        }
        
        spectrogram
    }
}
