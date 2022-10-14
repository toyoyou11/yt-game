mod material;
mod mesh;
mod vertex;

pub use material::*;
pub use mesh::*;
pub use vertex::*;

#[derive(Debug)]
pub struct ModelDescriptor<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, T> {
    pub name: &'a str,
    pub meshes: &'b [&'c MeshDescriptor<'f, 'g, 'h, T>],
    pub materials: &'d [&'e MaterialDescriptor<'i>],
}

#[derive(Debug)]
pub struct Model {
    pub name: String,
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn new<T: Vertex>(device: &wgpu::Device, desc: &ModelDescriptor<T>) -> Self {
        let name = desc.name.into();
        let meshes = desc
            .meshes
            .iter()
            .map(|desc| Mesh::new(device, desc))
            .collect::<Vec<_>>();
        let materials = desc
            .materials
            .iter()
            .map(|desc| Material::new(device, desc))
            .collect::<Vec<_>>();
        Self {
            name,
            meshes,
            materials,
        }
    }
}

pub trait DrawModel<'a> {
    fn draw_mesh(&mut self, mesh: &'a Mesh, material: &'a Material);
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: &'a Material,
        instances: std::ops::Range<u32>,
    );
    fn draw_model(&mut self, model: &'a Model);
    fn draw_model_instanced(&mut self, model: &'a Model, instances: std::ops::Range<u32>);
}

impl<'a, 'b, T: wgpu::util::RenderEncoder<'a>> DrawModel<'b> for T
where
    'b: 'a,
{
    fn draw_mesh(&mut self, mesh: &'b Mesh, material: &'b Material) {
        self.draw_mesh_instanced(mesh, material, 0..1);
    }
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'b Mesh,
        material: &'b Material,
        instances: std::ops::Range<u32>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), mesh.index_format);
        self.set_bind_group(0, &material.bind_group, &[]);
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }
    fn draw_model(&mut self, model: &'b Model) {
        self.draw_model_instanced(model, 0..1);
    }
    fn draw_model_instanced(&mut self, model: &'b Model, instances: std::ops::Range<u32>) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone());
        }
    }
}
