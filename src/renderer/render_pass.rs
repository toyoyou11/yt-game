mod camera_uniform_buffer;
mod light_uniform_buffer;
use super::*;
use std::sync::Arc;

use camera_uniform_buffer::*;
use light_uniform_buffer::*;

#[derive(Debug)]
pub struct RenderPass {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    camera_buffer: CameraUniformBuffer,
    light_buffer: LightUniformBuffer,
    pipeline: wgpu::RenderPipeline,
    instance_buffer: instance::InstanceBuffer,
    depth_texture: texture::Texture,
    max_point_lights: u32,
}

impl RenderPass {
    const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
    pub async fn new(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let instance_buffer = instance::InstanceBuffer::new(device.as_ref(), 100);
        let camera_buffer = CameraUniformBuffer::new(device.as_ref());
        let max_point_lights = 32;
        let light_buffer = LightUniformBuffer::new(device.as_ref(), max_point_lights);
        let depth_texture = Texture::create_depth_texture(device.as_ref(), width, height, Self::DEPTH_FORMAT, wgpu::CompareFunction::GreaterEqual);

        let material_layout = device.create_bind_group_layout(&model::Material::LAYOUT);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&material_layout, &camera_buffer.bind_group_layout, &light_buffer.bind_group_layout],
            push_constant_ranges: &[],
        });
        let shader = resource::load_shader("gl_shader.wgsl", device.as_ref())
            .await
            .unwrap();
        use model::Vertex;

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[model::ModelVertex::desc(), instance::InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState{
                format: Self::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        Self {
            device,
            queue,
            camera_buffer,
            light_buffer, 
            depth_texture,
            max_point_lights,
            pipeline,
            instance_buffer,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32){
        self.depth_texture = Texture::create_depth_texture(self.device.as_ref(), width, height, Self::DEPTH_FORMAT, wgpu::CompareFunction::GreaterEqual);
    }

    pub fn render(&mut self, scene: &Scene, view: &wgpu::TextureView) {
        self.update_camera_uniforms(scene.get_camera());
        self.update_light_uniforms(scene.get_lights());


        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor{
            label: Some("Render Encoder")
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor{
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment{
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations{
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_bind_group(1, &self.camera_buffer.bind_group, &[]);
            render_pass.set_bind_group(2, &self.light_buffer.bind_group, &[]);
            render_pass.set_pipeline(&self.pipeline);
            self.instance_buffer.clear();
            let num_entities = scene.iter_entities().len();
            if num_entities > self.instance_buffer.capacity() as usize{
                self.instance_buffer = instance::InstanceBuffer::new(self.device.as_ref(), 2 * num_entities as wgpu::BufferAddress);
            }
            for (_, entity) in scene.iter_entities(){
                let instance_raw = instance::InstanceRaw{
                    model: ( entity.position.to_homogeneous() * entity.scale.to_homogeneous() ).into()
                };
                let instance_slice = self.instance_buffer.insert(&instance_raw, self.queue.as_ref()).unwrap();
                render_pass.set_vertex_buffer(1, instance_slice);
                render_pass.draw_model(&entity.model);
            }

        }

        // present surface
        self.queue.submit(std::iter::once(encoder.finish()));
    }

    fn update_camera_uniforms(&self, camera: &camera::Camera){
        let camera_uniforms = CameraUniforms{
            view: camera.build_veiw_matrix().into(), 
            proj: camera.build_rev_projection_matrix().into()};
        self.camera_buffer.update(self.queue.as_ref(),camera_uniforms);
    }
    fn update_light_uniforms(&self, lights: &light::Lights){
        let ambient = AmbientLightUniforms::new(lights.ambient_light);
        let directional = DirectionalLightUniforms::new(lights.directional_light);
        let point_lights = lights.point_lights.iter().map(|l| {PointLightUniforms::new(*l)}).collect::<Vec<_>>();

        self.light_buffer.update(self.queue.as_ref(), ambient, directional, &point_lights);
    }
}
