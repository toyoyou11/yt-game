use super::*;
use crate::math::*;
/// Manager for mouse input
#[derive(Debug)]
pub struct MouseInputManager {
    delta_pos: (Float, Float),
    scroll: (Float, Float),
    buttons: [ElementState; Self::NUM_MOUSE_BUTTONS],
    previous_buttons: [ElementState; Self::NUM_MOUSE_BUTTONS],
}

impl MouseInputManager {
    const NUM_MOUSE_BUTTONS: usize = 52;
    /// Create default manager
    pub fn new() -> Self {
        Self {
            delta_pos: (0.0, 0.0),
            scroll: (0.0, 0.0),
            buttons: [ElementState::Up; Self::NUM_MOUSE_BUTTONS],
            previous_buttons: [ElementState::Up; Self::NUM_MOUSE_BUTTONS],
        }
    }

    pub fn get_scroll(&self) -> (Float, Float) {
        self.scroll
    }
    pub fn get_move(&self) -> (Float, Float) {
        self.delta_pos
    }
    /// This function should be called every frame
    pub fn begin_frame(&mut self) {
        self.previous_buttons = self.buttons;
        self.delta_pos = (0.0, 0.0);
        self.scroll = (0.0, 0.0);
    }

    pub fn is_down(&self, id: usize) -> bool {
        if id >= Self::NUM_MOUSE_BUTTONS {
            return false;
        }
        self.buttons[id] == ElementState::Down
    }

    pub fn is_up(&self, id: usize) -> bool {
        if id >= Self::NUM_MOUSE_BUTTONS {
            return false;
        }
        self.buttons[id] == ElementState::Up
    }
    pub fn is_pressed(&self, id: usize) -> bool {
        if id >= Self::NUM_MOUSE_BUTTONS {
            return false;
        }
        self.previous_buttons[id] == ElementState::Up && self.buttons[id] == ElementState::Down
    }

    pub fn is_released(&self, id: usize) -> bool {
        if id >= Self::NUM_MOUSE_BUTTONS {
            return false;
        }
        self.previous_buttons[id] == ElementState::Down && self.buttons[id] == ElementState::Up
    }
    /// process input and update
    pub fn process_input(&mut self, event: &winit::event::DeviceEvent) {
        use winit::event::DeviceEvent;
        match event {
            // process mouse motion
            DeviceEvent::MouseMotion { delta } => {
                self.delta_pos = (delta.0 as Float, delta.1 as Float);
            }
            // process mouse wheel motion
            DeviceEvent::MouseWheel { delta } => {
                if let winit::event::MouseScrollDelta::LineDelta(x, y) = delta {
                    self.scroll = (*x as Float, *y as Float);
                }
            }
            // process mouse button
            DeviceEvent::Button { button, state } => {
                let id = *button as usize;
                if id >= Self::NUM_MOUSE_BUTTONS {
                    return;
                }
                use winit::event;
                match state {
                    event::ElementState::Pressed => self.buttons[id] = ElementState::Down,
                    event::ElementState::Released => self.buttons[id] = ElementState::Up,
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn mouse_input_test() {
        let mut manager = MouseInputManager::new();
        manager.begin_frame();
        for i in 0..MouseInputManager::NUM_MOUSE_BUTTONS {
            assert_eq!(manager.buttons[i], ElementState::Up);
            assert_eq!(manager.previous_buttons[i], ElementState::Up);
            assert_eq!(manager.delta_pos, (0.0, 0.0));
            assert_eq!(manager.scroll, (0.0, 0.0));
        }
        use winit::event;
        let scroll = event::MouseScrollDelta::LineDelta(1.0, 2.0);
        manager.process_input(&event::DeviceEvent::MouseWheel { delta: scroll });
        manager.process_input(&event::DeviceEvent::MouseMotion { delta: (3.0, 4.0) });
        assert_eq!(manager.delta_pos, (3.0, 4.0));
        assert_eq!(manager.scroll, (1.0, 2.0));
        manager.process_input(&event::DeviceEvent::Button {
            button: 0,
            state: event::ElementState::Pressed,
        });
        assert_eq!(manager.buttons[0], ElementState::Down);
        assert_eq!(manager.is_pressed(0), true);

        manager.begin_frame();
        assert_eq!(manager.delta_pos, (0.0, 0.0));
        assert_eq!(manager.scroll, (0.0, 0.0));
        assert_eq!(manager.buttons[0], ElementState::Down);
        assert_eq!(manager.is_pressed(0), false);
        assert_eq!(manager.is_released(0), false);

        manager.begin_frame();
        manager.process_input(&event::DeviceEvent::Button {
            button: 0,
            state: event::ElementState::Released,
        });
        assert_eq!(manager.buttons[0], ElementState::Up);
        assert_eq!(manager.previous_buttons[0], ElementState::Down);
        assert_eq!(manager.is_released(0), true);
    }
}
