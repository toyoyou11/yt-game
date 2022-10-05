#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniforms {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}

#[derive(Debug)]
pub struct CameraUniformBuffer {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl CameraUniformBuffer {
    pub fn new(device: &wgpu::Device) -> Self {
        let size = std::mem::size_of::<CameraUniforms>() as wgpu::BufferAddress;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Camera Uniform Buffer"),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::all(),
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(size),
                },
                count: None,
            }],
            label: Some("Camera Bind Group Layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Camera Bind Group"),
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, data: CameraUniforms) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]))
    }
}
