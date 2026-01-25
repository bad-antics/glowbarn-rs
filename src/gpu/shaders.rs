// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! Additional compute shaders

/// Anomaly detection shader
pub const ANOMALY_SHADER: &str = r#"
struct Params {
    threshold: f32,
    window_size: u32,
    padding: vec2<u32>,
}

@group(0) @binding(0) var<storage, read> input_data: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(256)
fn detect_anomalies(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let n = arrayLength(&input_data);
    
    if (idx >= n) {
        return;
    }
    
    let half_window = params.window_size / 2u;
    let start = max(0u, idx - half_window);
    let end = min(n, idx + half_window + 1u);
    
    // Calculate local mean
    var sum: f32 = 0.0;
    var count: u32 = 0u;
    for (var i: u32 = start; i < end; i = i + 1u) {
        sum = sum + input_data[i];
        count = count + 1u;
    }
    let mean = sum / f32(count);
    
    // Calculate local variance
    var var_sum: f32 = 0.0;
    for (var i: u32 = start; i < end; i = i + 1u) {
        let diff = input_data[i] - mean;
        var_sum = var_sum + diff * diff;
    }
    let variance = var_sum / f32(count);
    let std_dev = sqrt(variance);
    
    // Z-score
    let z_score = abs(input_data[idx] - mean) / max(std_dev, 0.0001);
    
    // Output anomaly score
    output[idx] = select(0.0, z_score, z_score > params.threshold);
}
"#;

/// Correlation shader
pub const CORRELATION_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> signal_a: array<f32>;
@group(0) @binding(1) var<storage, read> signal_b: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(256)
fn cross_correlate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let lag = i32(global_id.x);
    let n_a = i32(arrayLength(&signal_a));
    let n_b = i32(arrayLength(&signal_b));
    let max_lag = i32(arrayLength(&output)) / 2;
    
    if (lag >= max_lag * 2) {
        return;
    }
    
    let actual_lag = lag - max_lag;
    
    var sum: f32 = 0.0;
    var count: i32 = 0;
    
    for (var i: i32 = 0; i < n_a; i = i + 1) {
        let j = i + actual_lag;
        if (j >= 0 && j < n_b) {
            sum = sum + signal_a[i] * signal_b[j];
            count = count + 1;
        }
    }
    
    output[u32(lag)] = select(0.0, sum / f32(count), count > 0);
}
"#;

/// Spectrogram shader
pub const SPECTROGRAM_SHADER: &str = r#"
const PI: f32 = 3.14159265359;

struct SpectrogramParams {
    fft_size: u32,
    hop_size: u32,
    num_frames: u32,
    num_bins: u32,
}

@group(0) @binding(0) var<storage, read> input_data: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<uniform> params: SpectrogramParams;

// Hann window function
fn hann_window(i: u32, n: u32) -> f32 {
    let x = f32(i) / f32(n - 1u);
    return 0.5 * (1.0 - cos(2.0 * PI * x));
}

@compute @workgroup_size(16, 16)
fn compute_spectrogram(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let frame = global_id.x;
    let bin = global_id.y;
    
    if (frame >= params.num_frames || bin >= params.num_bins) {
        return;
    }
    
    let start = frame * params.hop_size;
    let k = bin;
    
    // DFT for this frame and frequency bin
    var real: f32 = 0.0;
    var imag: f32 = 0.0;
    
    let n = params.fft_size;
    let n_f32 = f32(n);
    let k_f32 = f32(k);
    
    for (var j: u32 = 0u; j < n; j = j + 1u) {
        let idx = start + j;
        if (idx < arrayLength(&input_data)) {
            let window = hann_window(j, n);
            let sample = input_data[idx] * window;
            
            let angle = -2.0 * PI * k_f32 * f32(j) / n_f32;
            real = real + sample * cos(angle);
            imag = imag + sample * sin(angle);
        }
    }
    
    // Magnitude (log scale)
    let magnitude = sqrt(real * real + imag * imag);
    let log_mag = 20.0 * log(max(magnitude, 0.0001)) / log(10.0);
    
    // Store result
    let out_idx = frame * params.num_bins + bin;
    output[out_idx] = log_mag;
}
"#;

/// Statistics shader
pub const STATISTICS_SHADER: &str = r#"
struct Stats {
    mean: f32,
    variance: f32,
    min_val: f32,
    max_val: f32,
    sum: f32,
    count: u32,
    padding: vec2<u32>,
}

@group(0) @binding(0) var<storage, read> input_data: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: Stats;

var<workgroup> shared_sum: atomic<f32>;
var<workgroup> shared_min: atomic<f32>;
var<workgroup> shared_max: atomic<f32>;
var<workgroup> shared_count: atomic<u32>;

@compute @workgroup_size(256)
fn compute_statistics(@builtin(global_invocation_id) global_id: vec3<u32>, @builtin(local_invocation_id) local_id: vec3<u32>) {
    let idx = global_id.x;
    let n = arrayLength(&input_data);
    
    // Initialize shared variables
    if (local_id.x == 0u) {
        atomicStore(&shared_count, 0u);
    }
    workgroupBarrier();
    
    if (idx < n) {
        let val = input_data[idx];
        
        // Parallel reduction would be better, but this is simpler
        atomicAdd(&shared_count, 1u);
    }
    
    workgroupBarrier();
    
    // Only first thread computes final stats
    if (idx == 0u) {
        var sum: f32 = 0.0;
        var min_v: f32 = 1e30;
        var max_v: f32 = -1e30;
        
        for (var i: u32 = 0u; i < n; i = i + 1u) {
            let val = input_data[i];
            sum = sum + val;
            min_v = min(min_v, val);
            max_v = max(max_v, val);
        }
        
        let mean = sum / f32(n);
        
        // Second pass for variance
        var var_sum: f32 = 0.0;
        for (var i: u32 = 0u; i < n; i = i + 1u) {
            let diff = input_data[i] - mean;
            var_sum = var_sum + diff * diff;
        }
        let variance = var_sum / f32(n);
        
        output.mean = mean;
        output.variance = variance;
        output.min_val = min_v;
        output.max_val = max_v;
        output.sum = sum;
        output.count = n;
    }
}
"#;

/// Thermal colormap shader
pub const COLORMAP_SHADER: &str = r#"
// Inferno colormap approximation
fn inferno(t: f32) -> vec3<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    
    let r = clamp(
        -4.545831 * t3 + 5.014482 * t2 + 0.490997 * t - 0.003583,
        0.0, 1.0
    );
    let g = clamp(
        2.067913 * t3 - 2.861322 * t2 + 1.338326 * t - 0.024927,
        0.0, 1.0
    );
    let b = clamp(
        -2.213146 * t3 + 3.008929 * t2 + 0.099815 * t + 0.162531,
        0.0, 1.0
    );
    
    return vec3<f32>(r, g, b);
}

// Viridis colormap approximation
fn viridis(t: f32) -> vec3<f32> {
    let t2 = t * t;
    let t3 = t2 * t;
    
    let r = clamp(
        -1.330461 * t3 + 1.802813 * t2 + 0.260424 * t + 0.267004,
        0.0, 1.0
    );
    let g = clamp(
        -0.622971 * t3 + 0.425097 * t2 + 0.683031 * t + 0.004025,
        0.0, 1.0
    );
    let b = clamp(
        2.413464 * t3 - 3.761044 * t2 + 1.184967 * t + 0.329415,
        0.0, 1.0
    );
    
    return vec3<f32>(r, g, b);
}

@group(0) @binding(0) var<storage, read> thermal_data: array<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var<uniform> dims: vec2<u32>;

@compute @workgroup_size(16, 16)
fn apply_colormap(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= dims.x || y >= dims.y) {
        return;
    }
    
    let idx = y * dims.x + x;
    let value = thermal_data[idx];
    
    // Normalize to 0-1 (assuming temperature range -10 to 50 C)
    let normalized = clamp((value + 10.0) / 60.0, 0.0, 1.0);
    
    // Apply colormap
    let color = inferno(normalized);
    
    textureStore(output_texture, vec2<i32>(i32(x), i32(y)), vec4<f32>(color, 1.0));
}
"#;
