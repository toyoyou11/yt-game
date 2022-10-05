use std::mem;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}
impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct InstanceBuffer {
    capacity: wgpu::BufferAddress,
    size: AtomicU64,
    buffer: wgpu::Buffer,
}

impl InstanceBuffer {
    pub fn new(device: &wgpu::Device, capacity: wgpu::BufferAddress) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Buffer"),
            size: capacity * mem::size_of::<InstanceRaw>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let size = AtomicU64::new(0);

        Self {
            buffer,
            size,
            capacity,
        }
    }
    pub fn insert(&self, data: &InstanceRaw, queue: &wgpu::Queue) -> Option<wgpu::BufferSlice> {
        self.insert_slice(&[*data], queue)
    }

    pub fn insert_slice(
        &self,
        data: &[InstanceRaw],
        queue: &wgpu::Queue,
    ) -> Option<wgpu::BufferSlice> {
        let size = self.size.load(std::sync::atomic::Ordering::Relaxed);
        let len = data.len() as wgpu::BufferAddress;
        if size + len <= self.capacity {
            let s = size * mem::size_of::<InstanceRaw>() as wgpu::BufferAddress;
            queue.write_buffer(&self.buffer, s, bytemuck::cast_slice(data));

            let e = (size + len) * mem::size_of::<InstanceRaw>() as wgpu::BufferAddress;
            self.size
                .store(size + len, std::sync::atomic::Ordering::Relaxed);
            Some(self.buffer.slice(s..e))
        } else {
            None
        }
    }

    pub fn capacity(&self) -> wgpu::BufferAddress {
        self.capacity
    }

    pub fn clear(&mut self) {
        self.size.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug)]
struct InstanceBuffers {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    buffers: Vec<InstanceBuffer>,
    capacity: wgpu::BufferAddress,
}

impl InstanceBuffers {
    pub fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        capacity: wgpu::BufferAddress,
    ) -> Self {
        let buffer = InstanceBuffer::new(device.as_ref(), capacity);
        let mut buffers = Vec::new();
        buffers.push(buffer);

        Self {
            device,
            queue,
            buffers,
            capacity,
        }
    }
}
