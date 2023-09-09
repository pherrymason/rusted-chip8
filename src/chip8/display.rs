use macroquad::color::{BLACK, WHITE};
use macroquad::prelude::draw_rectangle;
use macroquad::window::clear_background;

pub struct Display {
    scale: f32,
    screen: Vec<bool>
}

impl Display {
    pub fn new() -> Display {
        Display {
            scale: 3.0,
            screen: vec![false; 64*32],
        }
    }
    pub fn clear(&mut self) {
        clear_background(BLACK);
        self.screen = vec![false; 64*32];
        println!("CLEAR");
    }

    pub fn draw(&mut self, x: u8, y: u8) -> bool {
        println!("DRAW {} {}",x,y);
        let active = !self.screen[ (y as usize *64) + x as usize];

        draw_rectangle(
            x as f32 * self.scale,
            y as f32 * self.scale,
            self.scale,
            self.scale,
            if active {WHITE} else {BLACK},
        );

        return active;
    }
}