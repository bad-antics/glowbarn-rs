// Copyright (c) 2026 bad-antics
// Licensed under the MIT License. See LICENSE file in the project root.
// https://github.com/bad-antics/glowbarn-rs

//! GPU buffer management

use anyhow::Result;
use wgpu::util::DeviceExt;

/// Ring buffer for streaming GPU data
pub struct GpuRingBuffer {
    buffer: wgpu::Buffer,
    size: u64,
    head: u64,
    capacity: u64,
}

impl GpuRingBuffer {
    pub fn new(device: &wgpu::Device, capacity: u64) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ring Buffer"),
            size: capacity,
            usage: wgpu::BufferUsages::STORAGE 
                | wgpu::BufferUsages::COPY_SRC 
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        
        Self {
            buffer,
            size: 0,
            head: 0,
            capacity,
        }
    }
    
    pub fn push(&mut self, queue: &wgpu::Queue, data: &[u8]) {
        let len = data.len() as u64;
        
        if len > self.capacity {
            return;  // Data too large
        }
        
        // Write data (wrapping if necessary)
        let space_at_end = self.capacity - self.head;
        
        if len <= space_at_end {
            queue.write_buffer(&self.buffer, self.head, data);
            self.head = (self.head + len) % self.capacity;
        } else {
            // Split write
            let first_part = space_at_end as usize;
            queue.write_buffer(&self.buffer, self.head, &data[..first_part]);
            queue.write_buffer(&self.buffer, 0, &data[first_part..]);
            self.head = len - space_at_end;
        }
        
        self.size = (self.size + len).min(self.capacity);
    }
    
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
    
    pub fn len(&self) -> u64 {
        self.size
    }
    
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
}

/// Double buffer for async compute
pub struct DoubleBuffer {
    buffers: [wgpu::Buffer; 2],
    current: usize,
    size: u64,
}

impl DoubleBuffer {
    pub fn new(device: &wgpu::Device, size: u64) -> Self {
        let buffers = [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Double Buffer A"),
                size,
                usage: wgpu::BufferUsages::STORAGE 
                    | wgpu::BufferUsages::COPY_SRC 
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Double Buffer B"),
                size,
                usage: wgpu::BufferUsages::STORAGE 
                    | wgpu::BufferUsages::COPY_SRC 
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ];
        
        Self {
            buffers,
            current: 0,
            size,
        }
    }
    
    pub fn current(&self) -> &wgpu::Buffer {
        &self.buffers[self.current]
    }
    
    pub fn previous(&self) -> &wgpu::Buffer {
        &self.buffers[1 - self.current]
    }
    
    pub fn swap(&mut self) {
        self.current = 1 - self.current;
    }
    
    pub fn write_current(&self, queue: &wgpu::Queue, data: &[u8]) {
        queue.write_buffer(&self.buffers[self.current], 0, data);
    }
}

/// Uniform buffer for shader parameters
pub struct UniformBuffer<T: bytemuck::Pod> {
    buffer: wgpu::Buffer,
    _marker: std::marker::PhantomData<T>,
}

impl<T: bytemuck::Pod> UniformBuffer<T> {
    pub fn new(device: &wgpu::Device, data: &T) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::bytes_of(data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        Self {
            buffer,
            _marker: std::marker::PhantomData,
        }
    }
    
    pub fn update(&self, queue: &wgpu::Queue, data: &T) {
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(data));
    }
    
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

/// Texture for 2D data (thermal images, spectrograms)
pub struct Texture2D {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    size: wgpu::Extent3d,
}

impl Texture2D {
    pub fn new(device: &wgpu::Device, width: u32, height: u32, format: wgpu::TextureFormat) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture2D"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING 
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });
        
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        Self { texture, view, size }
    }
    
    pub fn update(&self, queue: &wgpu::Queue, data: &[u8], bytes_per_row: u32) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(self.size.height),
            },
            self.size,
        );
    }
    
    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
    
    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }
    
    pub fn width(&self) -> u32 {
        self.size.width
    }
    
    pub fn height(&self) -> u32 {
        self.size.height
    }
}
