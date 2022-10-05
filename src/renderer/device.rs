
pub async fn create_device_surface<W: raw_window_handle::HasRawWindowHandle>(window: &W, window_width: u32, window_height: u32) -> (Device, Surface){
    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe{instance.create_surface(window)};
    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions{
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        },
    ).await.expect("Failed to aquire an adapter");

    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor{
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    ).await.expect("Failed to aquire a device");

    let config = wgpu::SurfaceConfiguration{
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_supported_formats(&adapter)[0],
        width: window_width,
        height: window_height,
        present_mode: wgpu::PresentMode::Fifo,
    };
    surface.configure(&device, &config);
    (
        Device{
            instance,
            adapter,
            device,
            queue,
        },
        Surface{
            surface,
            config,
        }
    )
}
#[derive(Debug)]
pub struct Device {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}


#[derive(Debug)]
pub struct Surface {
    pub surface: wgpu::Surface,
    pub config: wgpu::SurfaceConfiguration,
}

impl Surface{
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32){
        self.config.width = width;
        self.config.height = height;
        self.reconfigure(device);
    }

    pub fn reconfigure(&self, device: &wgpu::Device) {
        self.surface.configure(device, &self.config);
    }
}
