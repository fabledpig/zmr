use engine::input_handler::{InputHandler, Key};
use winit::event::{ElementState, VirtualKeyCode};

pub fn translate_winit_keyboard_event(
    input_handler: &InputHandler,
    virtual_key_code: VirtualKeyCode,
    element_state: ElementState,
) {
    let key = match virtual_key_code {
        VirtualKeyCode::Key1 => Some(Key::K1),
        VirtualKeyCode::Key2 => Some(Key::K2),
        VirtualKeyCode::Key3 => Some(Key::K3),
        VirtualKeyCode::Key4 => Some(Key::K4),
        VirtualKeyCode::Key5 => Some(Key::K5),
        VirtualKeyCode::Key6 => Some(Key::K6),
        VirtualKeyCode::Key7 => Some(Key::K7),
        VirtualKeyCode::Key8 => Some(Key::K8),
        VirtualKeyCode::Key9 => Some(Key::K9),
        VirtualKeyCode::Key0 => Some(Key::K0),
        VirtualKeyCode::A => Some(Key::A),
        VirtualKeyCode::B => Some(Key::B),
        VirtualKeyCode::C => Some(Key::C),
        VirtualKeyCode::D => Some(Key::D),
        VirtualKeyCode::E => Some(Key::E),
        VirtualKeyCode::F => Some(Key::F),
        VirtualKeyCode::G => Some(Key::G),
        VirtualKeyCode::H => Some(Key::H),
        VirtualKeyCode::I => Some(Key::I),
        VirtualKeyCode::J => Some(Key::J),
        VirtualKeyCode::K => Some(Key::K),
        VirtualKeyCode::L => Some(Key::L),
        VirtualKeyCode::M => Some(Key::M),
        VirtualKeyCode::N => Some(Key::N),
        VirtualKeyCode::O => Some(Key::O),
        VirtualKeyCode::P => Some(Key::P),
        VirtualKeyCode::Q => Some(Key::Q),
        VirtualKeyCode::R => Some(Key::R),
        VirtualKeyCode::S => Some(Key::S),
        VirtualKeyCode::T => Some(Key::T),
        VirtualKeyCode::U => Some(Key::U),
        VirtualKeyCode::V => Some(Key::V),
        VirtualKeyCode::W => Some(Key::W),
        VirtualKeyCode::X => Some(Key::X),
        VirtualKeyCode::Y => Some(Key::Y),
        VirtualKeyCode::Z => Some(Key::Z),
        VirtualKeyCode::Left => Some(Key::LeftArrow),
        VirtualKeyCode::Up => Some(Key::LeftArrow),
        VirtualKeyCode::Right => Some(Key::RightArrow),
        VirtualKeyCode::Down => Some(Key::DownArrow),
        VirtualKeyCode::Back => Some(Key::Backspace),
        VirtualKeyCode::Return => Some(Key::Enter),
        VirtualKeyCode::Space => Some(Key::Space),
        VirtualKeyCode::LControl => Some(Key::LCtrl),
        VirtualKeyCode::RControl => Some(Key::RCtrl),
        VirtualKeyCode::LAlt => Some(Key::LAlt),
        VirtualKeyCode::RAlt => Some(Key::RAlt),
        _ => None,
    };

    if let Some(key) = key {
        input_handler.key_state_changed(key, element_state == ElementState::Pressed);
    }
}
