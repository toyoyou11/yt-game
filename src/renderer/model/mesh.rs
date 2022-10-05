use super::vertex::*;
#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub index_format: wgpu::IndexFormat,
    pub num_elements: u32,
    pub material: usize,
}

impl Mesh {
    pub fn new_with_index_u32<T: Vertex>(
        name: String,
        vertices: &[T],
        indices: &[u32],
        material: usize,
        device: &wgpu::Device,
    ) -> Self {
        use wgpu::util::DeviceExt;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&name),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&name),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let index_format = wgpu::IndexFormat::Uint32;
        let num_elements = indices.len() as u32;
        Self {
            name,
            vertex_buffer,
            index_buffer,
            index_format,
            num_elements,
            material,
        }
    }
    pub fn new_with_index_u16<T: Vertex>(
        name: String,
        vertices: &[T],
        indices: &[u16],
        material: usize,
        device: &wgpu::Device,
    ) -> Self {
        use wgpu::util::DeviceExt;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&name),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&name),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let index_format = wgpu::IndexFormat::Uint16;
        let num_elements = indices.len() as u32;
        Self {
            name,
            vertex_buffer,
            index_buffer,
            index_format,
            num_elements,
            material,
        }
    }
}
