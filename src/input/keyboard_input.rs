use super::*;

use winit::event;
pub type Keycode = event::VirtualKeyCode;

/// Manages keyboard input. begin_frame() shoud be called every frame.
#[derive(Debug)]
pub struct KeyboardInputManager {
    previous_keys: [ElementState; Self::NUM_KEYS],
    keys: [ElementState; Self::NUM_KEYS],
}

impl KeyboardInputManager {
    /// Number of keys.
    const NUM_KEYS: usize = 256;
    /// Create manager with all keys up
    pub fn new() -> Self {
        Self {
            previous_keys: [ElementState::Up; Self::NUM_KEYS],
            keys: [ElementState::Up; Self::NUM_KEYS],
        }
    }

    /// This function should be called every frame
    pub fn begin_frame(&mut self) {
        self.previous_keys = self.keys;
    }

    /// receives device event and update
    pub fn process_input(&mut self, input: &winit::event::DeviceEvent) {
        if let winit::event::DeviceEvent::Key(input) = input {
            if let Some(keycode) = input.virtual_keycode {
                self.update(keycode, input.state);
            }
        }
    }

    pub fn is_down(&self, keycode: Keycode) -> bool {
        let code = keycode as usize;
        if code >= Self::NUM_KEYS {
            return false;
        }
        self.keys[code] == ElementState::Down
    }

    pub fn is_up(&self, keycode: Keycode) -> bool {
        let code = keycode as usize;
        if code >= Self::NUM_KEYS {
            return false;
        }
        self.keys[code] == ElementState::Up
    }
    /// This function returns true iff key was pressed in the frame
    pub fn is_pressed(&self, keycode: Keycode) -> bool {
        // make sure keycode is in range
        let keycode = keycode as usize;
        if keycode >= Self::NUM_KEYS {
            return false;
        }
        self.keys[keycode] == ElementState::Down && self.previous_keys[keycode] == ElementState::Up
    }

    /// This function returns true iff key was released in the frame
    pub fn is_released(&self, keycode: Keycode) -> bool {
        // make sure keycode is in range
        let keycode = keycode as usize;
        if keycode >= Self::NUM_KEYS {
            return false;
        }
        self.keys[keycode] == ElementState::Up && self.previous_keys[keycode] == ElementState::Down
    }

    fn update(&mut self, virtual_keycode: event::VirtualKeyCode, state: event::ElementState) {
        let code = virtual_keycode as usize;
        if code >= Self::NUM_KEYS {
            return;
        }
        match state {
            event::ElementState::Pressed => self.keys[code] = ElementState::Down,
            event::ElementState::Released => self.keys[code] = ElementState::Up,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn keyboard_input_test() {
        use winit::event;

        let mut manager = KeyboardInputManager::new();
        for i in 0..KeyboardInputManager::NUM_KEYS {
            assert_eq!(manager.previous_keys[i], ElementState::Up);
            assert_eq!(manager.keys[i], ElementState::Up);
        }

        manager.begin_frame();
        manager.update(event::VirtualKeyCode::A, event::ElementState::Pressed);
        assert_eq!(manager.is_pressed(Keycode::A), true);
        assert_eq!(
            manager.keys[event::VirtualKeyCode::A as usize],
            ElementState::Down
        );
        assert_eq!(
            manager.previous_keys[event::VirtualKeyCode::A as usize],
            ElementState::Up
        );

        manager.begin_frame();
        manager.update(event::VirtualKeyCode::B, event::ElementState::Pressed);
        assert_eq!(manager.is_pressed(Keycode::B), true);
        assert_eq!(
            manager.keys[event::VirtualKeyCode::B as usize],
            ElementState::Down
        );
        assert_eq!(
            manager.previous_keys[event::VirtualKeyCode::B as usize],
            ElementState::Up
        );
        assert_eq!(
            manager.keys[event::VirtualKeyCode::A as usize],
            ElementState::Down
        );
        assert_eq!(
            manager.previous_keys[event::VirtualKeyCode::A as usize],
            ElementState::Down
        );

        manager.begin_frame();
        manager.update(event::VirtualKeyCode::A, event::ElementState::Released);
        manager.update(event::VirtualKeyCode::B, event::ElementState::Released);
        assert_eq!(
            manager.keys[event::VirtualKeyCode::A as usize],
            ElementState::Up
        );
        assert_eq!(
            manager.previous_keys[event::VirtualKeyCode::A as usize],
            ElementState::Down
        );
        assert_eq!(manager.is_released(Keycode::A), true);
        assert_eq!(
            manager.keys[event::VirtualKeyCode::B as usize],
            ElementState::Up
        );
        assert_eq!(
            manager.previous_keys[event::VirtualKeyCode::B as usize],
            ElementState::Down
        );
        assert_eq!(manager.is_released(Keycode::B), true);
    }
}
