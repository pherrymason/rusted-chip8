use macroquad::color::{BLACK, WHITE};
use macroquad::prelude::draw_rectangle;
use macroquad::window::clear_background;

pub struct Display {
    scale: f32,
    screen: Vec<u8>,
}

const NATIVE_SCREEN_WIDTH: usize = 64;
const NATIVE_SCREEN_HEIGHT: usize = 32;

impl Display {
    pub fn new() -> Display {
        Display {
            scale: 10.0,
            screen: vec![0; NATIVE_SCREEN_WIDTH * NATIVE_SCREEN_HEIGHT],
        }
    }
    pub fn clear(&mut self) {
        clear_background(BLACK);
        self.screen = vec![0; NATIVE_SCREEN_WIDTH * NATIVE_SCREEN_HEIGHT];
    }

    pub fn draw(&mut self, x: u8, y: u8) -> bool {
        // Correct x and y
        let corrected_x = x % (NATIVE_SCREEN_WIDTH as u8 + 1);
        let corrected_y = y % (NATIVE_SCREEN_HEIGHT as u8 + 1);

        let pixel_coordinate: usize = (corrected_y as usize * (NATIVE_SCREEN_WIDTH - 1)) + corrected_x as usize;
        self.screen[pixel_coordinate] ^= 1;
        let active: u8 = self.screen[pixel_coordinate as usize] ^ 1;

        return if active == 1 { true } else { false };
    }

    pub fn render(&self) {
        let mut y = 0;
        while y < NATIVE_SCREEN_HEIGHT {
            let mut x = 0;
            while x < NATIVE_SCREEN_WIDTH {
                let pixel_coordinate = (y * (NATIVE_SCREEN_WIDTH - 1)) + x;
                let active = self.screen[pixel_coordinate as usize];
                draw_rectangle(
                    x as f32 * self.scale,
                    y as f32 * self.scale,
                    self.scale,
                    self.scale,
                    if active == 1 { WHITE } else { BLACK },
                );
                x += 1;
            }
            y += 1;
        }
    }
}