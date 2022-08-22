use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::renderer;
#[derive(Debug)]
pub struct Game{
    window: Window,
    renderer: renderer::Renderer,
}
#[derive(Debug)]
pub struct GameLauncher {
    event_loop: EventLoop<()>,
    game: Game,
}

impl Game{
    fn render(&mut self){
        self.renderer.render();
    }
}

impl GameLauncher {
    pub async fn new() -> Self {
        let (window, event_loop) = Self::create_window();
        let size = window.inner_size();
        let renderer = renderer::Renderer::new(&window, size.width, size.height).await;
        let game = Game{window, renderer};
        Self {
            event_loop,
            game,
        }
    }

    pub fn launch(mut self) {
        println!("Game started");
        let event_loop = self.event_loop;
        let mut game = self.game;
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
                Event::RedrawRequested(window_id) if window_id == game.window.id() => {
                    game.render();
                }
                _ => {}
            });
    }

    fn create_window() -> (Window, EventLoop<()>) {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().build(&event_loop).unwrap();
        (window, event_loop)
    }
}

