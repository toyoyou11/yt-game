mod world;
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
    entity: renderer::EntityId,
    rigid: physics::RigidBodyId,
}

impl Object{
    fn new(position: Isometry3, mut entity: renderer::Entity, mut rigid_body: physics::RigidBody, render_scene: &mut renderer::Scene, physics_world: &mut physics::PhysicsWorld) -> Self{
        entity.position = position;
        rigid_body.set_position(&position);
        let entity = render_scene.add_entity(entity);
        let rigid = physics_world.insert(rigid_body);
        Self{position, entity, rigid}
    }

    fn update(&mut self, render_scene: &mut renderer::Scene, world: &mut physics::PhysicsWorld) {
        self.position = *world.get(self.rigid).unwrap().get_position();
        render_scene.get_entity_mut(self.entity).unwrap().position = self.position;
    }
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
    rigid_scene: physics::PhysicsWorld,
    objects: Vec<Object>,
    now: instant::Instant,
}

impl Game{
    const FPS: Float = 165.0;
    async fn new(window: Window) -> Self{
        let size = window.inner_size();
        let renderer = renderer::Renderer::new(&window, size.width, size.height).await;
        let state = State::Active;
        let mut render_scene = renderer::Scene::new();
        let mut rigid_scene = physics::PhysicsWorld::new();
        let mut resource_manager = renderer.create_resource_manager();

        let objects = Self::create_objects(&mut resource_manager, &mut render_scene, &mut rigid_scene).await;
        let mut camera = renderer::camera::Camera::new();
        camera.position = Isometry3::look_at_lh(&Point3::new(0.0, 5.0, -10.0), &Point3::new(0.0, 0.0, 0.0), &Vector3::y_axis()).inverse();
        camera.aspect = size.width as Float / size.height as Float;
        render_scene.set_ambient_light(renderer::AmbientLight::new([0.1, 0.1, 0.1]));
        render_scene.get_directional_light_mut().direction = UnitVector3::new_normalize(Vector3::new(0.5, -1.0, 1.0));
        render_scene.set_camera(camera);
        Self{
            window,
            renderer,
            state,
            render_scene,
            rigid_scene,
            objects,
            now: instant::Instant::now(),
        }

    }

    async fn create_objects(resource: &mut renderer::ResourceManager, render_scene: &mut renderer::Scene, rigid_scene: &mut physics::PhysicsWorld) -> Vec<Object>{
        let mut objects = Vec::new();
        // ground
        let model = resource.get_model_json("white_cube.json").await.unwrap();
        let position = Isometry3::translation(0.0, 0.0, 0.0) * Isometry3::rotation(Vector3::new(0.0, 0.0, 0.0));
        let mut rigid_body = physics::RigidBody::new(physics::ShapeType::Cube(physics::Cube::new(Vector3::new(1000.0, 1.0, 1000.0))), 0.0);
        rigid_body.set_position(&position);
        let mut entity = renderer::Entity::new(model.clone());
        entity.position = position;
        entity.scale = Scale3::new(1000.0, 1.0, 1000.0);
        let obj2 = Object::new( position, entity, rigid_body , render_scene, rigid_scene);
        objects.push(obj2);
        // cube
        let model = resource.get_model_json("green_cube.json").await.unwrap();
        let position = Isometry3::translation(0.0, 10.0, 0.0) * Isometry3::rotation(Vector3::new(0.0, PI / 2.0, 0.0));
        let mut rigid_body = physics::RigidBody::new(physics::ShapeType::Cube(physics::Cube::new(Vector3::new(1.0, 1.0, 1.0))), 0.3);
        rigid_body.set_position(&position);
        rigid_body.set_linear_velocity(&Vector3::new(1.0, 0.0, 0.0));
        let mut entity = renderer::Entity::new(model.clone());
        entity.position = position;
        let obj1 = Object::new( position, entity, rigid_body , render_scene, rigid_scene);
        objects.push(obj1);

        // cube
        let model = resource.get_model_json("red_cube.json").await.unwrap();
        let position = Isometry3::translation(3.0, 5.0, 0.0) * Isometry3::rotation(Vector3::new(1.0, PI / 2.0, 0.0));
        let mut rigid_body = physics::RigidBody::new(physics::ShapeType::Cube(physics::Cube::new(Vector3::new(1.0, 1.0, 1.0))), 0.3);
        rigid_body.set_position(&position);
        rigid_body.set_linear_velocity(&Vector3::new(-0.0, 0.0, 0.0));
        let mut entity = renderer::Entity::new(model.clone());
        entity.position = position;
        let obj1 = Object::new( position, entity, rigid_body , render_scene, rigid_scene);
        objects.push(obj1);
        // ball
        let model = resource.get_ball_model("blue_material.json").await.unwrap();
        let position = Isometry3::translation(-5.0, 4.0, 0.0) * Isometry3::rotation(Vector3::new(1.0, PI / 2.0, 0.0));
        let mut rigid_body = physics::RigidBody::new(physics::ShapeType::Sphere(physics::Sphere::new(1.0)), 0.3);
        rigid_body.set_linear_velocity(&Vector3::new(0.3, 0.0, -0.4));
        rigid_body.set_position(&position);
        let mut entity = renderer::Entity::new(model.clone());
        entity.position = position;
        let obj1 = Object::new( position, entity, rigid_body , render_scene, rigid_scene);
        objects.push(obj1);


        objects
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
        let dt = self.wait(1.0 / Self::FPS);
        self.rigid_scene.update(dt);
        self.objects.iter_mut().for_each(|o| o.update(&mut self.render_scene, &mut self.rigid_scene));
    }

    fn resize(&mut self, width: u32, height: u32){
        self.renderer.resize(width, height);
        let mut camera = *self.render_scene.get_camera();
        camera.aspect = width as f32 / height as f32;
        self.render_scene.set_camera(camera);
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
    let mut game = Game::new(window).await;
    event_loop
        .run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == game.window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(physical_size) => {
                    game.resize(physical_size.width, physical_size.height);
                },
                WindowEvent::ScaleFactorChanged{new_inner_size, ..} =>{
                    game.resize(new_inner_size.width, new_inner_size.height);
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


