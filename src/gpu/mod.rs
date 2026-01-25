// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! GPU compute module using wgpu

mod shaders;
mod buffers;
mod pipelines;

pub use shaders::*;
pub use buffers::*;
pub use pipelines::*;

use anyhow::{anyhow, Result};
use std::sync::Arc;
use tracing::{info, warn, debug};

/// GPU compute context
pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_info: wgpu::AdapterInfo,
    entropy_pipeline: Option<EntropyPipeline>,
    fft_pipeline: Option<FftPipeline>,
}

impl GpuContext {
    /// Create GPU context
    pub async fn new() -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("No GPU adapter found"))?;
        
        let adapter_info = adapter.get_info();
        info!(
            "Using GPU: {} ({:?})",
            adapter_info.name,
            adapter_info.backend
        );
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("GlowBarn GPU"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;
        
        Ok(Self {
            device,
            queue,
            adapter_info,
            entropy_pipeline: None,
            fft_pipeline: None,
        })
    }
    
    /// Initialize compute pipelines
    pub fn init_pipelines(&mut self) -> Result<()> {
        self.entropy_pipeline = Some(EntropyPipeline::new(&self.device)?);
        self.fft_pipeline = Some(FftPipeline::new(&self.device)?);
        info!("GPU compute pipelines initialized");
        Ok(())
    }
    
    /// Get GPU info
    pub fn get_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }
    
    /// Compute entropy on GPU
    pub async fn compute_entropy(&self, data: &[f32]) -> Result<f32> {
        let pipeline = self.entropy_pipeline.as_ref()
            .ok_or_else(|| anyhow!("Entropy pipeline not initialized"))?;
        
        pipeline.compute(&self.device, &self.queue, data).await
    }
    
    /// Compute FFT on GPU
    pub async fn compute_fft(&self, data: &[f32]) -> Result<Vec<f32>> {
        let pipeline = self.fft_pipeline.as_ref()
            .ok_or_else(|| anyhow!("FFT pipeline not initialized"))?;
        
        pipeline.compute(&self.device, &self.queue, data).await
    }
    
    /// Batch compute entropy for multiple windows
    pub async fn compute_entropy_batch(&self, windows: &[Vec<f32>]) -> Result<Vec<f32>> {
        let mut results = Vec::with_capacity(windows.len());
        
        for window in windows {
            let entropy = self.compute_entropy(window).await?;
            results.push(entropy);
        }
        
        Ok(results)
    }
}

/// Entropy compute pipeline
pub struct EntropyPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl EntropyPipeline {
    pub fn new(device: &wgpu::Device) -> Result<Self> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Entropy Shader"),
            source: wgpu::ShaderSource::Wgsl(ENTROPY_SHADER.into()),
        });
        
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Entropy Bind Group Layout"),
            entries: &[
                // Input data buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Output buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Histogram buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Entropy Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Entropy Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "compute_entropy",
        });
        
        Ok(Self {
            pipeline,
            bind_group_layout,
        })
    }
    
    pub async fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[f32]) -> Result<f32> {
        use wgpu::util::DeviceExt;
        
        let n = data.len();
        if n == 0 {
            return Ok(0.0);
        }
        
        // Create input buffer
        let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Input Buffer"),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::STORAGE,
        });
        
        // Create output buffer
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: 4, // Single f32
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        
        // Create histogram buffer (256 bins)
        let histogram_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Histogram Buffer"),
            size: 256 * 4,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        
        // Create staging buffer for reading result
        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Buffer"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        
        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Entropy Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: output_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: histogram_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Create command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Entropy Encoder"),
        });
        
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Entropy Pass"),
                timestamp_writes: None,
            });
            
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((n as u32 + 255) / 256, 1, 1);
        }
        
        // Copy result to staging buffer
        encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, 4);
        
        queue.submit(Some(encoder.finish()));
        
        // Read result
        let buffer_slice = staging_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        
        device.poll(wgpu::Maintain::Wait);
        rx.recv()??;
        
        let data = buffer_slice.get_mapped_range();
        let result = bytemuck::cast_slice::<u8, f32>(&data)[0];
        
        drop(data);
        staging_buffer.unmap();
        
        Ok(result)
    }
}

/// FFT compute pipeline  
pub struct FftPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl FftPipeline {
    pub fn new(device: &wgpu::Device) -> Result<Self> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("FFT Shader"),
            source: wgpu::ShaderSource::Wgsl(FFT_SHADER.into()),
        });
        
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("FFT Bind Group Layout"),
            entries: &[
                // Input real buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Input imaginary buffer
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("FFT Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("FFT Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "compute_fft",
        });
        
        Ok(Self {
            pipeline,
            bind_group_layout,
        })
    }
    
    pub async fn compute(&self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[f32]) -> Result<Vec<f32>> {
        use wgpu::util::DeviceExt;
        
        let n = data.len();
        if n == 0 {
            return Ok(vec![]);
        }
        
        // Pad to power of 2
        let padded_len = n.next_power_of_two();
        let mut padded_data = data.to_vec();
        padded_data.resize(padded_len, 0.0);
        
        // Create real buffer
        let real_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FFT Real Buffer"),
            contents: bytemuck::cast_slice(&padded_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        
        // Create imaginary buffer (zeros)
        let imag_data = vec![0.0f32; padded_len];
        let imag_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("FFT Imag Buffer"),
            contents: bytemuck::cast_slice(&imag_data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        
        // Create staging buffers
        let staging_real = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Real"),
            size: (padded_len * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        
        let staging_imag = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Staging Imag"),
            size: (padded_len * 4) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        
        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("FFT Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: real_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: imag_buffer.as_entire_binding(),
                },
            ],
        });
        
        // Execute
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("FFT Encoder"),
        });
        
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("FFT Pass"),
                timestamp_writes: None,
            });
            
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.dispatch_workgroups((padded_len as u32 + 255) / 256, 1, 1);
        }
        
        encoder.copy_buffer_to_buffer(&real_buffer, 0, &staging_real, 0, (padded_len * 4) as u64);
        encoder.copy_buffer_to_buffer(&imag_buffer, 0, &staging_imag, 0, (padded_len * 4) as u64);
        
        queue.submit(Some(encoder.finish()));
        
        // Read results
        let real_slice = staging_real.slice(..);
        let imag_slice = staging_imag.slice(..);
        
        let (tx1, rx1) = std::sync::mpsc::channel();
        let (tx2, rx2) = std::sync::mpsc::channel();
        
        real_slice.map_async(wgpu::MapMode::Read, move |r| { tx1.send(r).unwrap(); });
        imag_slice.map_async(wgpu::MapMode::Read, move |r| { tx2.send(r).unwrap(); });
        
        device.poll(wgpu::Maintain::Wait);
        rx1.recv()??;
        rx2.recv()??;
        
        let real_data = real_slice.get_mapped_range();
        let imag_data = imag_slice.get_mapped_range();
        
        let real: &[f32] = bytemuck::cast_slice(&real_data);
        let imag: &[f32] = bytemuck::cast_slice(&imag_data);
        
        // Calculate magnitudes
        let magnitudes: Vec<f32> = real.iter()
            .zip(imag.iter())
            .map(|(r, i)| (r * r + i * i).sqrt())
            .take(padded_len / 2)  // Only first half is meaningful
            .collect();
        
        drop(real_data);
        drop(imag_data);
        staging_real.unmap();
        staging_imag.unmap();
        
        Ok(magnitudes)
    }
}

/// Entropy compute shader
const ENTROPY_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> input_data: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<storage, read_write> histogram: array<atomic<u32>>;

@compute @workgroup_size(256)
fn compute_entropy(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let idx = global_id.x;
    let n = arrayLength(&input_data);
    
    if (idx >= n) {
        return;
    }
    
    // Normalize value to 0-255 bin
    let value = input_data[idx];
    let min_val = -10.0;
    let max_val = 10.0;
    let normalized = clamp((value - min_val) / (max_val - min_val), 0.0, 1.0);
    let bin = u32(normalized * 255.0);
    
    // Increment histogram bin atomically
    atomicAdd(&histogram[bin], 1u);
    
    // Only thread 0 calculates final entropy
    if (idx == 0u) {
        workgroupBarrier();
        
        var entropy: f32 = 0.0;
        let n_f32 = f32(n);
        
        for (var i: u32 = 0u; i < 256u; i = i + 1u) {
            let count = f32(atomicLoad(&histogram[i]));
            if (count > 0.0) {
                let p = count / n_f32;
                entropy = entropy - p * log2(p);
            }
        }
        
        output[0] = entropy;
    }
}
"#;

/// FFT compute shader (simplified DFT for demo)
const FFT_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read_write> real: array<f32>;
@group(0) @binding(1) var<storage, read_write> imag: array<f32>;

const PI: f32 = 3.14159265359;

@compute @workgroup_size(256)
fn compute_fft(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let k = global_id.x;
    let n = arrayLength(&real);
    
    if (k >= n) {
        return;
    }
    
    // Simple DFT (not FFT, for demonstration)
    // In production, use proper FFT butterfly algorithm
    var sum_real: f32 = 0.0;
    var sum_imag: f32 = 0.0;
    
    let n_f32 = f32(n);
    let k_f32 = f32(k);
    
    for (var j: u32 = 0u; j < n; j = j + 1u) {
        let j_f32 = f32(j);
        let angle = -2.0 * PI * k_f32 * j_f32 / n_f32;
        sum_real = sum_real + real[j] * cos(angle) - imag[j] * sin(angle);
        sum_imag = sum_imag + real[j] * sin(angle) + imag[j] * cos(angle);
    }
    
    workgroupBarrier();
    
    real[k] = sum_real;
    imag[k] = sum_imag;
}
"#;
