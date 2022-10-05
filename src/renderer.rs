pub mod light;
pub mod camera;
pub mod resource;
pub mod scene;
pub mod model;
pub mod entity;
mod texture;
mod render_pass;
mod device;
mod instance;

use std::sync::Arc;
use model::DrawModel;

pub use self::{
    light::*, camera::*, resource::*, scene::*, model::*,entity::*,
};


use self::{
    texture::*, render_pass::*, device::*, instance::*
};


// lib.rs
#[derive(Debug)]
pub struct Renderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: device::Surface,
    render_pass: RenderPass,
}

impl Renderer {
    pub async fn new<W: raw_window_handle::HasRawWindowHandle>(window: &W, window_width: u32, window_height: u32) -> Self {
        let (device, surface) = device::create_device_surface(window, window_width, window_height).await;
        let queue = Arc::new(device.queue);
        let device = Arc::new(device.device);

        let render_pass = RenderPass::new(device.clone(), queue.clone(), surface.config.format, surface.config.width, surface.config.height).await;
        Self { device, queue, surface, render_pass}
    }

    pub fn create_resource_manager(&self) -> resource::ResourceManager{
        resource::ResourceManager::new(self.device.clone(), self.queue.clone())
    }
    pub fn render(&mut self, scene: &Scene) -> Result<(), wgpu::SurfaceError> {
        // get current texture of surface
        let output = self.surface.surface.get_current_texture()?;
        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );
        self.render_pass.render(scene, &view);
        output.present();

        // everything is fine
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32){
        self.surface.resize(self.device.as_ref(), width, height);
        self.render_pass.resize(width, height);
    }

    pub fn reconfigure_surface(&self){
        self.surface.reconfigure(self.device.as_ref());
    }
}
