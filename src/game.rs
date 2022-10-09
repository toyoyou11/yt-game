mod world;
use std::sync::Arc;
use crate::math::*;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use super::*;

#[derive(Debug)]
struct Object{
    position: Isometry3,
    entity: renderer::EntityIndex,
    rigid: physics::RigidBodyId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State{
    Active,
    Pose,
    Dead,
}

use crate::renderer;
#[derive(Debug)]
pub struct Game{
    window: Window,
    renderer: renderer::Renderer,
    state: State,
    render_scene: renderer::Scene,
    rigid_scene: physics::Scene,
    obj1: Object,
    obj2: Object,
    now: instant::Instant,
}

impl Game{
    async fn new(window: Window) -> Self{
        let size = window.inner_size();
        let renderer = renderer::Renderer::new(&window, size.width, size.height).await;
        let state = State::Active;
        let mut render_scene = renderer::Scene::new();
        let mut rigid_scene = physics::Scene::new();
        let mut resource_manager = renderer.create_resource_manager();

        let (obj1, obj2) = Self::create_objects(&mut resource_manager, &mut render_scene, &mut rigid_scene).await;
        let mut camera = renderer::camera::Camera::new();
        camera.position = Isometry3::look_at_lh(&Point3::new(0.0, 3.0, -10.0), &Point3::new(0.0, 0.0, 0.0), &Vector3::y_axis()).inverse();
        camera.aspect = size.width as Float / size.height as Float;
        render_scene.get_lights_mut().directional_light.direction = UnitVector3::new_normalize(Vector3::new(1.0, -1.0, -1.0));
        render_scene.set_camera(camera);
        Self{
            window,
            renderer,
            state,
            render_scene,
            rigid_scene,
            obj1,
            obj2,
            now: instant::Instant::now(),
        }

    }

    async fn create_objects(resource: &mut renderer::ResourceManager, render_scene: &mut renderer::Scene, rigid_scene: &mut physics::Scene) -> (Object, Object){
        let position = Isometry3::translation(0.0, 10.0, 0.0) * Isometry3::rotation(Vector3::new(0.0, std::f32::consts::PI, 0.0));
        let model = resource.get_model_json("cube_model.json").await.unwrap();
        let mut rigid_body = physics::RigidBody::new(physics::ShapeType::Sphere(physics::shape::Sphere::new(1.0)), 1.0);
        rigid_body.set_position(&position);
        let rigid = rigid_scene.insert(rigid_body);
        let mut entity = renderer::Entity::new(model.clone());
        entity.position = position;
        let entity = render_scene.add_entity(entity);
        let obj1 = Object{ position, rigid, entity };

        let position = Isometry3::translation(0.0, -100.0, 0.0);
        let mut rigid_body = physics::RigidBody::new(physics::ShapeType::Sphere(physics::shape::Sphere::new(100.0)), 0.0);
        rigid_body.set_position(&position);
        let rigid = rigid_scene.insert(rigid_body);
        let mut entity = renderer::Entity::new(model.clone());
        entity.position = position;
        entity.scale = Scale3::new(100.0, 100.0, 100.0);
        let entity = render_scene.add_entity(entity);
        let obj2 = Object{ position, rigid, entity };
        (obj1, obj2)
    }
    fn render(&mut self){
        match self.renderer.render(&self.render_scene){
            Ok(_) => {},
            Err(wgpu::SurfaceError::Lost) => self.renderer.reconfigure_surface(),
            Err(wgpu::SurfaceError::OutOfMemory) => {
                eprintln!("surface is out of memory");
                self.state = State::Dead;
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }

    fn update(&mut self) {
        let dt = self.wait(1.0 / 165.0);
        let rigid_body = self.rigid_scene.get_mut(self.obj1.rigid).unwrap();
        rigid_body.apply_force_world(&Vector3::new(0.0, -10.0, 0.0));
        self.rigid_scene.update(dt);
        let rigid_body = self.rigid_scene.get(self.obj1.rigid).unwrap();
        let entity = self.render_scene.get_entity_mut(self.obj1.entity).unwrap();
        entity.position = *rigid_body.get_position();
        let rigid_body = self.rigid_scene.get(self.obj2.rigid).unwrap();
        let entity = self.render_scene.get_entity_mut(self.obj2.entity).unwrap();
        entity.position = *rigid_body.get_position();
    }

    fn wait(&mut self, dt: Float) -> Float{
        let mut next = instant::Instant::now();
        while next.duration_since(self.now).as_secs_f32() < dt{
            next = instant::Instant::now();
        }
        let delta_time = next.duration_since(self.now).as_secs_f32();
        self.now = next;
        
        delta_time.min(1.0 / 15.0)
    }

    fn sweep(&mut self, control_flow: &mut ControlFlow){
        if self.state == State::Dead{
            *control_flow = ControlFlow::Exit;
        }
    }
}

pub async fn launch(window: Window, event_loop: EventLoop<()>) {
    println!("Game started");
    let mut game = Game::new(window).await;
    event_loop
        .run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == game.window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    game.renderer.resize(physical_size.width, physical_size.height);
                },
                WindowEvent::ScaleFactorChanged{new_inner_size, ..} =>{
                    game.renderer.resize(new_inner_size.width, new_inner_size.height);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                game.update();
                game.window.request_redraw();
            }
            Event::RedrawRequested(window_id) if window_id == game.window.id() => {
                game.render();
            }
            Event::RedrawEventsCleared => {
                game.sweep(control_flow);
            }
            _ => {}
        });
}


