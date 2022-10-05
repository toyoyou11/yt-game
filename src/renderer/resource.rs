use std::io::{BufReader, Cursor};
use std::sync::Arc;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use cfg_if::cfg_if;
use wgpu::util::DeviceExt;

use super::*;

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
        let default_texture = Arc::new(Texture::from_image(device.as_ref(), queue.as_ref(), &image, Some(Self::DEFAULT_TEXTURE_NAME)));
        let models = HashMap::new();
        textures.insert(Self::DEFAULT_TEXTURE_NAME.to_string(), default_texture);
        Self{device, queue, textures, models}
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
            let material_uniform = model::MaterialUniform{
                diffuse: material_data.diffuse,
                shininess: material_data.shininess,
                alpha: material_data.alpha,
                specular: material_data.specular,
            };
            let diffuse_texture = self.get_texture(&material_data.diffuse_texture.unwrap_or(Self::DEFAULT_TEXTURE_NAME.into())).await?;
            let specular_texture = self.get_texture(&material_data.specular_texture.unwrap_or("".to_string())).await.unwrap_or(diffuse_texture.clone());

            let material = model::Material::new(material_data.name.clone(), material_uniform, diffuse_texture, specular_texture, self.device.as_ref());
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

                    let mesh = model::Mesh::new_with_index_u32(m.name.clone(), &vertices, &m.indices, m.material_id, self.device.as_ref());
                    meshes.push(mesh);
                }
                MeshType::Skeletal(_) => {
                    unimplemented!();
                }
            }
        }

        Ok(Arc::new(model::Model{name: model_data.name, meshes, materials}))
    }
    pub async fn get_model_obj(&mut self, file_name: &str) -> anyhow::Result<Arc<Model>>{
    if let Some(m) = self.models.get(file_name){
        return Ok(m.clone());
    }
    let obj_text = load_string(file_name).await?;
    let obj_cursor = Cursor::new(obj_text);
    let mut obj_reader = BufReader::new(obj_cursor);

    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = Vec::new();

    for m in &obj_materials?{
            let diffuse_texture = self.get_texture(&m.diffuse_texture).await?;
            let specular_texture = self.get_texture(&m.specular_texture).await.unwrap_or(diffuse_texture.clone());
            let diffuse = m.diffuse;
            let shininess = m.shininess;
            let specular = m.specular;
            let alpha = 1.0;
            let uniform = model::MaterialUniform{diffuse, shininess, alpha, specular};

            let material = model::Material::new(
                m.name.clone(),
                uniform,
                diffuse_texture,
                specular_texture,
                self.device.as_ref()
            );
            materials.push(material);

    }
    let mut meshes = Vec::new();

    for m in models{
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| model::ModelVertex {
                    pos: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                })
                .collect::<Vec<_>>();

            let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Index Buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            let mesh = model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                index_format: wgpu::IndexFormat::Uint32,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
            };
            meshes.push(mesh);
    }


    Ok(Arc::new(Model { name: file_name.into(), meshes, materials}))
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
    diffuse: [f32; 3],
    alpha: f32,
    specular: [f32; 3],
    shininess: f32,
    diffuse_texture: Option<String>,
    specular_texture: Option<String>,
    normal: Option<String>,
}
