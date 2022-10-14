mod keyboard_input;
mod mouse_input;

use keyboard_input::KeyboardInputManager;
pub use keyboard_input::Keycode;
use mouse_input::MouseInputManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementState {
    Up,
    Down,
}
#[derive(Debug)]
pub struct InputManager {
    keyboard_manager: KeyboardInputManager,
    mouse_manager: MouseInputManager,
}

impl InputManager {
    /// Create input manager
    pub fn new() -> Self {
        Self {
            keyboard_manager: KeyboardInputManager::new(),
            mouse_manager: MouseInputManager::new(),
        }
    }

    pub fn begin_frame(&mut self) {
        self.keyboard_manager.begin_frame();
        self.mouse_manager.begin_frame();
    }

    pub fn process_input(&mut self, input: &winit::event::DeviceEvent) {
        self.mouse_manager.process_input(input);
        self.keyboard_manager.process_input(input);
    }

    pub fn key_is_up(&self, key: Keycode) -> bool {
        self.keyboard_manager.is_up(key)
    }
    pub fn key_is_down(&self, key: Keycode) -> bool {
        self.keyboard_manager.is_down(key)
    }
    pub fn key_is_pressed(&self, key: Keycode) -> bool {
        self.keyboard_manager.is_pressed(key)
    }
    pub fn key_is_released(&self, key: Keycode) -> bool {
        self.keyboard_manager.is_released(key)
    }

    pub fn button_is_up(&self, button: u16) -> bool {
        self.mouse_manager.is_up(button as usize)
    }
    pub fn button_is_down(&self, button: u16) -> bool {
        self.mouse_manager.is_down(button as usize)
    }
    pub fn button_is_pressed(&self, button: u16) -> bool {
        self.mouse_manager.is_pressed(button as usize)
    }
    pub fn button_is_released(&self, button: u16) -> bool {
        self.mouse_manager.is_released(button as usize)
    }

    pub fn get_scroll(&self) -> (f32, f32) {
        self.mouse_manager.get_scroll()
    }

    pub fn get_mouse_move(&self) -> (f32, f32) {
        self.mouse_manager.get_move()
    }
}
