use crate::chip8::Chip8;

mod chip8;

fn main() {
    let emulator: Chip8 = Chip8::new();
    println!("Hello, world!");
}
