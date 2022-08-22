mod device;
#[derive(Debug)]
pub struct Renderer {
    device: device::Device,
    surface: device::Surface,
}

impl Renderer {
    pub async fn new<W: raw_window_handle::HasRawWindowHandle>(window: &W, window_width: u32, window_height: u32) -> Self {
        let (device, surface) = device::create_device_surface(window, window_width, window_height).await;
        Self { device, surface }
    }
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // get current texture of surface
        let output = self.surface.surface.get_current_texture()?;
        let view = output.texture.create_view(
            &wgpu::TextureViewDescriptor::default()
        );

        let mut encoder = self.device.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder")
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment{
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations{
                        load: wgpu::LoadOp::Clear(wgpu::Color{
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        // present surface
        self.device.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        // everything is fine
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32){
        self.surface.resize(&self.device, width, height);
    }
}
