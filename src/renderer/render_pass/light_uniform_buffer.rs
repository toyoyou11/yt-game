use super::light::*;

use std::mem::size_of;
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct AmbientLightUniforms {
    pub color: [f32; 3],
    pub padding: u32,
}

impl AmbientLightUniforms {
    pub fn new(ambient: AmbientLight) -> Self {
        Self {
            color: ambient.color,
            padding: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct DirectionalLightUniforms {
    pub color: [f32; 3],
    pub padding1: u32,
    pub direction: [f32; 3],
    pub padding2: u32,
}

impl DirectionalLightUniforms {
    pub fn new(directional: DirectionalLight) -> Self {
        Self {
            color: directional.color,
            padding1: 0,
            direction: directional.direction.into_inner().into(),
            padding2: 0,
        }
    }
}
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
pub struct PointLightUniforms {
    pub color: [f32; 3],
    pub padding1: u32,
    pub pos: [f32; 3],
    pub padding2: u32,
    pub radius: f32,
    pub padding3: [u32; 3],
}

impl PointLightUniforms {
    pub fn new(point_light: PointLight) -> Self {
        Self {
            color: point_light.color,
            padding1: 0,
            pos: point_light.point.into(),
            padding2: 0,
            radius: point_light.radius,
            padding3: [0, 0, 0],
        }
    }
}

#[derive(Debug)]
pub struct LightUniformBuffer {
    pub ambient_buffer: wgpu::Buffer,
    pub directional_buffer: wgpu::Buffer,
    pub point_buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    pub max_point_lights: u32,
}

impl LightUniformBuffer {
    const DIRECTIONAL_SIZE: wgpu::BufferAddress =
        size_of::<DirectionalLightUniforms>() as wgpu::BufferAddress;
    const AMBIENT_SIZE: wgpu::BufferAddress =
        size_of::<AmbientLightUniforms>() as wgpu::BufferAddress;
    const POINT_SIZE: wgpu::BufferAddress = size_of::<PointLightUniforms>() as wgpu::BufferAddress;
    pub fn new(device: &wgpu::Device, max_point_lights: u32) -> Self {
        let point_size = size_of::<[u32; 4]>() as wgpu::BufferAddress
            + Self::POINT_SIZE * max_point_lights as wgpu::BufferAddress;

        let ambient_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Ambient Light Uniform Buffer"),
            size: Self::AMBIENT_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let directional_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Directional Light Uniform Buffer"),
            size: Self::DIRECTIONAL_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let point_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Directional Light Uniform Buffer"),
            size: point_size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Uniform Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(Self::AMBIENT_SIZE),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(Self::DIRECTIONAL_SIZE),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Uniform Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &ambient_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &directional_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &point_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });
        Self {
            ambient_buffer,
            directional_buffer,
            point_buffer,
            bind_group_layout,
            bind_group,
            max_point_lights,
        }
    }
    pub fn update(
        &self,
        queue: &wgpu::Queue,
        ambient: AmbientLightUniforms,
        directional: DirectionalLightUniforms,
        points: &[PointLightUniforms],
    ) {
        queue.write_buffer(&self.ambient_buffer, 0, bytemuck::bytes_of(&ambient));
        queue.write_buffer(
            &self.directional_buffer,
            0,
            bytemuck::bytes_of(&directional),
        );
        let num_points = points.len() as u32;
        queue.write_buffer(&self.point_buffer, 0, bytemuck::bytes_of(&num_points));
        queue.write_buffer(
            &self.point_buffer,
            size_of::<[u32; 4]>() as wgpu::BufferAddress,
            bytemuck::cast_slice(points),
        );
    }
}
