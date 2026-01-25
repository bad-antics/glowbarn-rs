//! GPU compute pipelines

use anyhow::Result;
use std::sync::Arc;

/// Pipeline manager for creating and caching compute pipelines
pub struct PipelineManager {
    device: Arc<wgpu::Device>,
    pipelines: std::collections::HashMap<String, wgpu::ComputePipeline>,
    layouts: std::collections::HashMap<String, wgpu::BindGroupLayout>,
}

impl PipelineManager {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self {
            device,
            pipelines: std::collections::HashMap::new(),
            layouts: std::collections::HashMap::new(),
        }
    }
    
    /// Create or get cached pipeline
    pub fn get_or_create_pipeline(
        &mut self,
        name: &str,
        shader_source: &str,
        entry_point: &str,
        layout: &wgpu::BindGroupLayout,
    ) -> &wgpu::ComputePipeline {
        if !self.pipelines.contains_key(name) {
            let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(name),
                source: wgpu::ShaderSource::Wgsl(shader_source.into()),
            });
            
            let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Layout", name)),
                bind_group_layouts: &[layout],
                push_constant_ranges: &[],
            });
            
            let pipeline = self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(name),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: entry_point,
            });
            
            self.pipelines.insert(name.to_string(), pipeline);
        }
        
        self.pipelines.get(name).unwrap()
    }
    
    /// Create bind group layout for common patterns
    pub fn create_storage_layout(&self, num_buffers: usize, read_only: &[bool]) -> wgpu::BindGroupLayout {
        let entries: Vec<_> = (0..num_buffers)
            .map(|i| {
                wgpu::BindGroupLayoutEntry {
                    binding: i as u32,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: read_only.get(i).copied().unwrap_or(false),
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            })
            .collect();
        
        self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Storage Layout"),
            entries: &entries,
        })
    }
}

/// Batch compute dispatcher
pub struct BatchDispatcher {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl BatchDispatcher {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self { device, queue }
    }
    
    /// Dispatch multiple compute operations
    pub fn dispatch_batch(
        &self,
        operations: &[ComputeOperation],
    ) -> wgpu::SubmissionIndex {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Batch Encoder"),
        });
        
        for op in operations {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: op.label.as_deref(),
                timestamp_writes: None,
            });
            
            pass.set_pipeline(&op.pipeline);
            pass.set_bind_group(0, &op.bind_group, &[]);
            pass.dispatch_workgroups(op.workgroups.0, op.workgroups.1, op.workgroups.2);
        }
        
        self.queue.submit(Some(encoder.finish()))
    }
}

/// Single compute operation
pub struct ComputeOperation<'a> {
    pub label: Option<String>,
    pub pipeline: &'a wgpu::ComputePipeline,
    pub bind_group: &'a wgpu::BindGroup,
    pub workgroups: (u32, u32, u32),
}

/// Async result reader
pub struct ResultReader {
    device: Arc<wgpu::Device>,
}

impl ResultReader {
    pub fn new(device: Arc<wgpu::Device>) -> Self {
        Self { device }
    }
    
    /// Read buffer contents asynchronously
    pub async fn read_buffer<T: bytemuck::Pod>(&self, staging_buffer: &wgpu::Buffer, count: usize) -> Result<Vec<T>> {
        let slice = staging_buffer.slice(..);
        
        let (tx, rx) = tokio::sync::oneshot::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        
        self.device.poll(wgpu::Maintain::Wait);
        rx.await??;
        
        let data = slice.get_mapped_range();
        let result: Vec<T> = bytemuck::cast_slice(&data).to_vec();
        
        drop(data);
        staging_buffer.unmap();
        
        Ok(result.into_iter().take(count).collect())
    }
}
