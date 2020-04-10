use super::game::GAME_WRAPPER;
use luminance_glfw::{Action, Key};
use rutie::{AnyObject, Boolean, Class, Module, Object, Symbol};
use std::collections::BTreeMap;

wrappable_struct!(InputInner, InputWrapper, INPUT_WRAPPER);

module!(Input);

pub struct InputInner {
    pub input: BTreeMap<String, KeyState>,
}

#[derive(Clone, Copy, Debug)]
pub enum KeyState {
    Released,
    Pressed(u64),
}

#[rustfmt::skip]
methods!(
    Input,
    _itself,

    fn key_state(k: Symbol) -> Symbol {
        let inner = _itself.instance_variable_get("@input");
        Symbol::new(match inner.get_data(&*INPUT_WRAPPER).check_key(k.unwrap().to_string()) {
            KeyState::Released => "released",
            KeyState::Pressed(frame) if frame == get_current_frame() => "hit",
            KeyState::Pressed(_) => "down",
        })
    }

    fn key_down(k: Symbol) -> Boolean {
        let inner = _itself.instance_variable_get("@input");
        Boolean::new(inner.get_data(&*INPUT_WRAPPER).is_key_down(k.unwrap().to_string()))
    }

    fn key_hit(k: Symbol) -> Boolean {
        let inner = _itself.instance_variable_get("@input");
        Boolean::new(inner.get_data(&*INPUT_WRAPPER).is_key_hit(k.unwrap().to_string()))
    }
);

impl InputInner {
    fn new() -> Self {
        InputInner {
            input: BTreeMap::new(),
        }
    }

    fn check_key(&self, key: String) -> KeyState {
        self.input.get(&key).copied().unwrap_or(KeyState::Released)
    }

    fn is_key_down(&self, key: String) -> bool {
        match self.check_key(key) {
            KeyState::Released => false,
            KeyState::Pressed(_) => true,
        }
    }

    fn is_key_hit(&self, key: String) -> bool {
        match self.check_key(key) {
            KeyState::Released => false,
            KeyState::Pressed(frame) => {
                let current_frame = get_current_frame();
                frame == current_frame
            }
        }
    }

    pub fn handle_key_event(&mut self, key: Key, action: Action) {
        if let Some(name) = get_key_name(key) {
            match action {
                Action::Press => {
                    let current_frame = get_current_frame();
                    self.input.insert(name, KeyState::Pressed(current_frame));
                }
                Action::Release => {
                    self.input.insert(name, KeyState::Released);
                }
                Action::Repeat => {}
            }
        }
    }
}

fn get_current_frame() -> u64 {
    Module::from_existing("Game")
        .instance_variable_get("@inner")
        .get_data(&*GAME_WRAPPER)
        .time
        .frame
}

fn get_key_name(key: Key) -> Option<String> {
    key.get_name()
        .map(|s| s.to_lowercase())
        .or_else(|| {
            match key {
                Key::Escape => Some("escape"),
                Key::Up => Some("up"),
                Key::Down => Some("down"),
                Key::Left => Some("left"),
                Key::Right => Some("right"),
                Key::Num1 => Some("num1"),
                Key::Num2 => Some("num2"),
                Key::Num3 => Some("num3"),
                Key::Num4 => Some("num4"),
                Key::Num5 => Some("num5"),
                Key::Num6 => Some("num6"),
                Key::Num7 => Some("num7"),
                Key::Num8 => Some("num8"),
                Key::Num9 => Some("num9"),
                Key::Num0 => Some("num0"),
                _ => {
                    println!("Unrecognized key: {:?}", key);
                    None
                }
            }
            .map(|s| s.to_string())
        })
        .map(|s| {
            match s.as_str() {
                "1" => "_1",
                "2" => "_2",
                "3" => "_3",
                "4" => "_4",
                "5" => "_5",
                "6" => "_6",
                "7" => "_7",
                "8" => "_8",
                "9" => "_9",
                "0" => "_0",
                _ => &s,
            }
            .to_string()
        })
}

pub fn add_input_module() {
    let mut module = Module::new("Input");

    let inner: AnyObject =
        Class::from_existing("Object").wrap_data(InputInner::new(), &*INPUT_WRAPPER);
    module.instance_variable_set("@input", inner);

    module.def_self("key", key_state);
    module.def_self("key_down", key_down);
    module.def_self("key_hit", key_hit);
}
