use super::Chip8;
use rand::Rng;
use crate::chip8::{V_SIZE};

impl Chip8 {
    pub fn opcode_clear_screen(&mut self) {
        self.clear_screen = true;
    }

    pub fn return_from_subroutine(&mut self) {
        self.pc = self.stack.pop();
        self.stack_pointer -= 1;
    }

    pub fn set_schip_graphic_mode(&mut self) {}

    pub fn opcode_jmp(&mut self, address: u16) {
        self.pc = address;
    }

    pub fn opcode_call_subroutine(&mut self, address: u16) {
        self.stack.push(self.pc + 2);
        self.pc = address;
    }
    pub fn opcode_skip_if_vx_equals_nn(&mut self, x: u8, nn: u8) {
        if self.v[x as usize] == nn {
            self.increment_pc();
        }
    }
    pub fn opcode_skip_if_vx_diffs_nn(&mut self, x: u8, nn: u8) {
        if self.v[x as usize] != nn {
            self.increment_pc();
        }
    }
    pub fn opcode_skip_if_vx_equals_vy(&mut self, x: u8, y: u8) {
        if self.v[x as usize] == self.v[y as usize] {
            self.increment_pc();
        }
    }
    pub fn opcode_set_vx_to_nn(&mut self, x: u8, nn: u8) {
        self.v[x as usize] = nn;
    }
    pub fn opcode_adds_nn_to_vx(&mut self, x: u8, nn: u8) {
        self.v[x as usize] = (self.v[x as usize] + nn) & 0xFF;
    }
    pub fn opcode_set_vx_to_vy(&mut self, opcode: u16, x: usize, y: usize) {
        match opcode & 0x0F {
            0 => self.v[x] = self.v[y],
            1 => self.v[x] |= self.v[y],
            2 => self.v[x] &= self.v[y],
            3 => self.v[x] ^= self.v[y],
            4 => {
                // 8XY4	Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't.
                let sum: u16 = self.v[y] as u16 + self.v[x] as u16;
                if sum > 0xFF {
                    self.v[0xF] = 1; // Carry
                } else {
                    self.v[0xF] = 0; // No carry
                }
                self.v[x] = self.v[x].wrapping_add(self.v[y]);
            }
            5 => {
                // 8XY5	VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't.
                //self.v[0xF] = if self.v[y] > self.v[x] {0} else {1};
                self.v[0xF] = if self.v[x] >= self.v[y] { 1 } else { 0 };
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);
            }
            6 => {
                // 8XY6	Shifts VX right by one. VF is set to the value of the least significant bit of VX before the shift.[2]
                self.v[0xF] = self.v[x] & 0x01;
                self.v[x] >>= 1;
            }
            7 => {
                // 8XY7	Sets VX to (VY minus VX). VF is set to 0 when there's a borrow, and 1 when there isn't.
                self.v[0xF] = if self.v[y] < self.v[x] { 0 } else { 1 };
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);
            }
            0xE => {
                self.v[0xF] = self.v[x] >> 7;
                self.v[x] <<= 1;
            }
            _ => { panic!("unhandled opcode") }
        }
    }

    // Skips the next instruction if VX doesn't equal VY.
    pub fn opcode_skips_if_vx_diffs_vy(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.increment_pc();
        }
    }
    pub fn opcode_set_i_to_nnn(&mut self, nnn: u16) {
        self.address_register = nnn;
    }
    pub fn opcode_jmp_nnn_plus_v0(&mut self, nnn: u16) {
        self.pc = nnn + (self.v[0] as u16);
    }
    pub fn opcode_set_vx_random(&mut self, x: usize, nn: u8) {
        let random_number: u8 = self.rng.gen_range(0..=255);
        self.v[x] = random_number;
    }
    pub fn opcode_draw(&mut self, x: usize, y: usize, n: u8) {
        self.v[0xF] = if self.draw_sprite(self.v[x], self.v[y], self.address_register, n) { 1 } else { 0 };
    }
    pub fn draw_sprite(&mut self, x: u8, y: u8, i: u16, n: u8) -> bool {
        return true;
    }
    pub fn opcode_skip_key_pressed_in_vx(&mut self, x: usize) {
        if self.keypad.status(self.v[x]) == 1 {
            self.increment_pc();
        }
    }
    pub fn opcode_skip_key_not_pressed_in_vx(&mut self, x: usize) {
        if self.keypad.status(self.v[x]) == 0 {
            self.increment_pc();
        }
    }
    pub fn opcode_save_delay_to_vx(&mut self, x: usize) {
        self.v[x] = self.timer_delay;
    }
    pub fn opcode_wait_key(&mut self, x: usize) {
        let mut keypress = false;
        for i in 0..=V_SIZE {
            if self.keypad.status(i as u8) != 0 {
                self.v[x] = i as u8;
                keypress = true
            }
        }

        //if (!keypress) {
        //    self.pc -= 2; // force try again
        //}
    }
    pub fn opcode_save_vx_to_delay(&mut self, x: usize) {
        self.timer_delay = self.v[x];
    }
    pub fn opcode_save_vx_to_sound_timer(&mut self, x: usize) {
        self.timer_sound = self.v[x];
    }
    pub fn opcode_adds_vx_to_i(&mut self, x: usize) {
        self.address_register += self.v[x] as u16;
        self.v[0xF] = if (self.address_register + self.v[x] as u16) > 0xFFF { 1 } else { 0 };
    }

    // I is set the address for the hexadecimal character sprite referred to by the register VX 5 chars high
    pub fn opcode_set_i_with_vx(&mut self, x: usize) {
        self.address_register = (self.v[x] * 5) as u16;
    }

    // FX33	Stores the Binary-coded decimal representation of VX, with the most significant of three digits at the address in I, the middle digit at I plus 1, and the least significant digit at I plus 2. (In other words, take the decimal representation of VX, place the hundreds digit in memory at location in I, the tens digit at location I+1, and the ones digit at location I+2.)
    pub fn opcode_save_bin_vx(&mut self, x: usize) {
        self.memory[self.address_register as usize] = self.v[x] / 100;
        self.memory[self.address_register as usize + 1] = (self.v[x] / 10) % 10;
        self.memory[self.address_register as usize + 2] = (self.v[x] % 100) % 10;
    }
    pub fn opcode_dump_v_to_memory(&mut self, x: usize) {
        let mut i: usize = 0;
        while i <= x {
            self.memory[self.address_register as usize + i] = self.v[i];
            i += 1;
        }
        self.address_register += x as u16 + 1;
    }

    // FX65	Fills V0 to VX with values from memory starting at address I.[4]
    pub fn opcode_fill_v_with_memory(&mut self, x: usize) {
        let mut i: usize = 0;
        while i <= x {
            self.v[self.address_register as usize] = (self.address_register + i as u16) as u8
        }
        self.address_register += x as u16 + 1;
    }
}