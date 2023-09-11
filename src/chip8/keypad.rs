pub struct Keypad {
    key_states: [u8; 16],
}

impl Keypad {
    pub fn new() -> Keypad {
        Keypad {
            key_states: [0; 16],
        }
    }
    pub fn press(&mut self, key_index: usize) {
        self.key_states[key_index] = 1;
    }

    pub fn release(&mut self, key_index: usize) {
        self.key_states[key_index] = 0;
    }

    pub fn status(&self, key_index: usize) -> u8 {
        return self.key_states[key_index];
    }
}