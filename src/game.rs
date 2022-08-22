use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
#[derive(Debug)]
pub struct Game {
    window: Window,
    event_loop: EventLoop<()>,
}

impl Game {
    pub fn new() -> Self {
        let (window, event_loop) = Self::create_window();
        Self { window, event_loop }
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
