use std::io::{BufReader, Cursor};
use std::sync::Arc;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

use super::*;

#[cfg(target_arch="wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug)]
pub struct ResourceManager{
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    textures: HashMap<String, Arc<Texture>>,
    models: HashMap<String, Arc<Model>>,
}

impl ResourceManager{
    const DEFAULT_TEXTURE_NAME: &'static str = "default_texture";
    const DEFAULT_TEXUTER_WIDTH: u32 = 128;
    const DEFAULT_TEXUTER_HEIGHT: u32 = 128;
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self{
        let mut textures = HashMap::new();
        let image = image::DynamicImage::ImageRgba8(
        image::RgbaImage::from_fn(Self::DEFAULT_TEXUTER_WIDTH, Self::DEFAULT_TEXUTER_HEIGHT, |x, y| {
            image::Rgba([255, 255, 255, 255])
        })
        );
        let default_texture = Arc::new(Texture::from_image(device.as_ref(), queue.as_ref(), &image, Some(&Self::DEFAULT_TEXTURE_NAME)));
        let models = HashMap::new();
        textures.insert(Self::DEFAULT_TEXTURE_NAME.to_string(), default_texture);
        Self{device, queue, textures, models}
    }

    pub fn create_material(&self, desc: &MaterialDescriptor) -> Material{
        Material::new(self.device.as_ref(), desc)
    }

    pub fn create_mesh<T: Vertex>(&self, desc: &MeshDescriptor<T>) -> Mesh{
        Mesh::new(self.device.as_ref(), desc)
    }

    pub fn get_default_texture(&self) -> Arc<Texture> {
        self.textures.get(Self::DEFAULT_TEXTURE_NAME).unwrap().clone()
    }

    pub fn create_model<'a, 'b, 'c, T: Vertex>(&mut self, desc: &ModelDescriptor<T>) -> Arc<Model>{
        let model = Arc::new(Model::new(self.device.as_ref(), desc));
        self.models.insert(desc.name.into(), model.clone());
        model
    }

    pub async fn get_ball_model(&mut self, material_file: &str) -> anyhow::Result<Arc<Model>>{
        let (vertices, indices) = crate::renderer::ball_generator::generate_ball();
        let meshes = vec![model::Mesh::new_with_index_u32(self.device.as_ref(), "ball".into(), &vertices, &indices, 0)];
        let material_json = load_string(material_file).await?;
        let material_data: MaterialData = serde_json::from_str(&material_json)?;
        let materials = vec![self.create_material_from_data(material_data).await?];
        Ok(Arc::new(Model{name: "ball".into(), meshes, materials}))
    }

    pub async fn get_model_json(&mut self, file_name: &str) -> anyhow::Result<Arc<Model>>{
        if let Some(m) = self.models.get(file_name){
            return Ok(m.clone());
        }
        let json_text = load_string(file_name).await?;
        let model_data: ModelData = serde_json::from_str(&json_text)?;

        let mut materials = Vec::with_capacity(model_data.materials.len());

        // Load materials.
        for m in &model_data.materials{
            let material_json = load_string(m).await?;
            let material_data: MaterialData = serde_json::from_str(&material_json)?;
            let material = self.create_material_from_data(material_data).await?;
            materials.push(material);
        }

        // Load meshes.
        let mut meshes = Vec::new();
        let meshes_json = load_string(&model_data.meshes).await?;
        let meshes_data: MeshesData = serde_json::from_str(&meshes_json)?;
        for m in &meshes_data.meshes{
            match &m.vertices{
                MeshType::Static(vertices_data) => {
                    let vertices = vertices_data.iter().map(|v| {
                        model::ModelVertex{pos: v.pos, tex_coords: v.coords, normal: v.normal}
                    }).collect::<Vec<_>>();

                    let mesh = model::Mesh::new_with_index_u32(self.device.as_ref(),m.name.clone(), &vertices, &m.indices, m.material_id );
                    meshes.push(mesh);
                }
                MeshType::Skeletal(_) => {
                    unimplemented!();
                }
            }
        }

        Ok(Arc::new(model::Model{name: model_data.name, meshes, materials}))
    }
    async fn create_material_from_data(&mut self, data: MaterialData) -> anyhow::Result<Material>{
        let texture = self.get_texture(&data.texture.unwrap_or(Self::DEFAULT_TEXTURE_NAME.into())).await?;
        let desc = MaterialDescriptor{
            name: &data.name,
            albedo: data.albedo,
            alpha: data.alpha,
            roughness: data.roughness,
            metalic: data.metalic,
            texture,
        };

            Ok(Material::new(self.device.as_ref(), &desc))
    }

    pub async fn get_texture(&mut self, file_name: &str) -> anyhow::Result<Arc<Texture>>{
        match self.textures.get(file_name){
            Some(texture) => Ok(texture.clone()),
            None => {
                let texture = Arc::new(load_texture(file_name, self.device.as_ref(), self.queue.as_ref()).await?);
                self.textures.insert(file_name.into(), texture.clone());
                Ok(texture)
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn format_url(file_name: &str) -> reqwest::Url {
    let window = web_sys::window().unwrap();
    let location = window.location();
    let base = reqwest::Url::parse(&format!(
        "{}/{}/",
        location.origin().unwrap(),
        option_env!("RES_PATH").unwrap_or("res"),
    )).unwrap();
    base.join(file_name).unwrap()
}

pub async fn load_string(file_name: &str) -> anyhow::Result<String> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let txt = reqwest::get(url)
                .await?
                .text()
                .await?;
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let txt = std::fs::read_to_string(path)?;
        }
    }

    Ok(txt)
}

pub async fn load_shader(file_name: &str, device: &wgpu::Device) -> anyhow::Result<wgpu::ShaderModule>{
    let shader_str = load_string(file_name).await?;
    Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor{
        label: Some(file_name),
        source: wgpu::ShaderSource::Wgsl(shader_str.into())
    }))
}

pub async fn load_binary(file_name: &str) -> anyhow::Result<Vec<u8>> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            let url = format_url(file_name);
            let data = reqwest::get(url)
                .await?
                .bytes()
                .await?
                .to_vec();
        } else {
            let path = std::path::Path::new(env!("OUT_DIR"))
                .join("res")
                .join(file_name);
            let data = std::fs::read(path)?;
        }
    }

    Ok(data)
}


pub async fn load_texture(file_name: &str, device: &wgpu::Device, queue: &wgpu::Queue,) -> anyhow::Result<Texture>{
    let data = load_binary(file_name).await?;
    Texture::from_bytes(device, queue, &data, file_name)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelData{
    name: String,
    meshes: String,
    materials: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshesData{
    meshes: Vec<MeshData>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MeshData{
    name: String,
    vertices: MeshType,
    indices: Vec<u32>,
    material_id: usize,
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
struct StaticVertexData{
    pos: [f32; 3],
    coords: [f32; 2],
    normal: [f32; 3],
}
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, bytemuck::Pod, bytemuck::Zeroable)]
struct SkeletalVertexData{
    pos: [f32; 3],
    coords: [f32; 2],
    normal: [f32; 3],
    bones: [u32; 4],
    weight: [f32; 4],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MeshType{
    Static(Vec<StaticVertexData>),
    Skeletal(Vec<SkeletalVertexData>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaterialData{
    name: String,
    albedo: [f32; 3],
    alpha: f32,
    roughness: f32,
    metalic: f32,
    texture: Option<String>,
}
