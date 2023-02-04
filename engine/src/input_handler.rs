use std::sync::Mutex;
use std::sync::MutexGuard;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use util::{internal_mut_struct, smart_enum};

smart_enum!(
    pub, Key, K0, K1, K2, K3, K4, K5, K6, K7, K8, K9, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O,
    P, Q, R, S, T, U, V, W, X, Y, Z, UpArrow, DownArrow, LeftArrow, RightArrow, Shift, LCtrl,
    RCtrl, LAlt, RAlt, Enter, Backspace, Tab, Space
);

#[derive(Clone, Copy, Eq, PartialEq)]
enum KeyState {
    Up,
    Down(Instant),
}

struct InputHandlerImpl {
    key_state_map: HashMap<Key, KeyState>,
    now: Instant,
}

impl InputHandlerImpl {
    pub fn new() -> Self {
        let mut key_state_map = HashMap::new();
        for key in Key::values() {
            key_state_map.insert(key, KeyState::Up);
        }

        Self {
            key_state_map,
            now: Instant::now(),
        }
    }

    pub fn key_state_changed(&mut self, key: Key, pressed: bool) {
        let key_state = self.key_state_map.get_mut(&key).unwrap();
        *key_state = match *key_state {
            KeyState::Up => {
                if pressed {
                    KeyState::Down(self.now)
                } else {
                    *key_state
                }
            }
            KeyState::Down(_) => {
                if pressed {
                    *key_state
                } else {
                    KeyState::Up
                }
            }
        };
    }
}

internal_mut_struct!(InputHandler, InputHandlerImpl);

impl InputHandler {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(InputHandlerImpl::new()),
        }
    }

    pub fn update(&self, delta_time: Duration) {
        self.lock_inner().now += delta_time;
    }

    pub fn key_state_changed(&self, key: Key, pressed: bool) {
        self.lock_inner().key_state_changed(key, pressed);
    }

    pub fn is_pressed(&self, key: Key) -> bool {
        let key_state = *self.lock_inner().key_state_map.get(&key).unwrap();
        key_state != KeyState::Up
    }

    pub fn is_held(&self, key: Key) -> bool {
        const HOLD_DURATION: Duration = Duration::from_secs(1);

        let inner = self.lock_inner();
        let key_state = inner.key_state_map.get(&key).unwrap();
        if let KeyState::Down(pressed_since) = *key_state {
            inner.now - pressed_since > HOLD_DURATION
        } else {
            false
        }
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
