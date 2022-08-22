use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::renderer;
#[derive(Debug)]
pub struct Game {
    window: Window,
    event_loop: EventLoop<()>,
    renderer: renderer::Renderer,
}

impl Game {
    pub async fn new() -> Self {
        let (window, event_loop) = Self::create_window();
        let size = window.inner_size();
        let renderer = renderer::Renderer::new(&window, size.width, size.height).await;
        Self {
            window,
            event_loop,
            renderer,
        }
    }

    pub fn start(mut self) {
        println!("Game started");
        self.event_loop
            .run(move |event, _, control_flow| match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(physical_size) => {
                        self.renderer.resize(physical_size.width, physical_size.height);
                    },
                    WindowEvent::ScaleFactorChanged{new_inner_size, ..} =>{
                        self.renderer.resize(new_inner_size.width, new_inner_size.height);
                    }
                    _ => {}
                },
                _ => {}
            });
    }

    fn create_window() -> (Window, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        (window, event_loop)
    }
}
