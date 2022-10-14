use super::super::texture::Texture;
use std::sync::Arc;

#[derive(Debug)]
pub struct MaterialDescriptor<'a> {
    pub name: &'a str,
    pub albedo: [f32; 3],
    pub alpha: f32,
    pub roughness: f32,
    pub metalic: f32,
    pub texture: Arc<Texture>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialUniform {
    pub albedo: [f32; 3],
    pub alpha: f32,
    pub roughness: f32,
    pub metalic: f32,
    pub padding: [u32; 2],
}

#[derive(Debug)]
pub struct Material {
    pub name: String,
    pub buffer: wgpu::Buffer,
    pub texture: Arc<Texture>,
    pub bind_group: wgpu::BindGroup,
}

impl Material {
    pub const LAYOUT: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<MaterialUniform>() as u64
                    ),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
        label: Some("material bind group layout"),
    };

    pub fn new<'a>(device: &wgpu::Device, desc: &MaterialDescriptor<'a>) -> Self {
        let uniform = MaterialUniform {
            albedo: desc.albedo,
            alpha: desc.alpha,
            roughness: desc.roughness,
            metalic: desc.metalic,
            padding: [0, 0],
        };
        let layout = device.create_bind_group_layout(&Self::LAYOUT);
        use wgpu::util::DeviceExt;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(desc.name),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&desc.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&desc.texture.sampler),
                },
            ],
            label: None,
        });
        Self {
            name: desc.name.into(),
            buffer,
            bind_group,
            texture: desc.texture.clone(),
        }
    }
}
