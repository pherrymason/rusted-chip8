use std::fs;
use macroquad::prelude::next_frame;
use crate::chip8::Chip8;

mod chip8;

#[macroquad::main("Rusted Chip8")]
async fn main() {
    let mut emulator: Chip8 = Chip8::new();

    let result = fs::read("roms/Particle Demo [zeroZshadow, 2008].ch8");
    //let result = fs::read("roms/Space Invaders [David Winter].ch8");
    let program: Vec<u8>;
    match result {
        Ok(contents) => {
            program = contents.into();
        },
        Err(e) => {
            panic!("Could not open rom");
        }
    }
    println!("Rom opened: len {}", program.len());
    emulator.load(program);
    emulator.start();
    loop {
        emulator.run();
        next_frame().await;
    }
}
