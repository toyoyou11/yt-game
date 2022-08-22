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
        todo!()
    }

    pub fn resize(&mut self, width: u32, height: u32){
        self.surface.resize(&self.device, width, height);
    }
}
