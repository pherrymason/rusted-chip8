use super::Chip8;
use rand::Rng;
use crate::chip8::{V_SIZE};

impl Chip8 {
    pub fn opcode_clear_screen(&mut self) {
        self.display.clear();
    }

    pub fn return_from_subroutine(&mut self) {
        self.pc = self.stack.pop();
        self.skip_increment_pc = true;
    }

    pub fn set_schip_graphic_mode(&mut self) {}

    pub fn opcode_jmp(&mut self, address: u16) {
        self.pc = address;
        self.skip_increment_pc = true;
    }

    pub fn opcode_call_subroutine(&mut self, address: u16) {
        self.stack.push(self.pc + 2);
        self.pc = address;
        self.skip_increment_pc = true;
    }
    pub fn opcode_skip_if_vx_equals_nn(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.increment_pc();
        }
    }
    pub fn opcode_skip_if_vx_diffs_nn(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.increment_pc();
        }
    }
    pub fn opcode_skip_if_vx_equals_vy(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.increment_pc();
        }
    }
    pub fn opcode_set_vx_to_nn(&mut self, x: usize, nn: u8) {
        self.v[x] = nn;
    }
    pub fn opcode_adds_nn_to_vx(&mut self, x: usize, nn: u8) {
        self.v[x] = self.v[x].wrapping_add(nn);
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
                self.v[0xF] = if self.v[y] >= self.v[x] { 1 } else { 0 };
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);
            }
            0xE => {
                self.v[0xF] = (self.v[x] >> 7) & 0x1;
                self.v[x] <<= 1;
            }
            _ => { panic!("unhandled opcode") }
        }
    }

    // Skips the next instruction if VX doesn't equal VY.
    pub fn opcode_skips_if_vx_diffs_vy(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.increment_pc();
            self.skip_increment_pc = true;
        }
    }
    pub fn opcode_set_i_to_nnn(&mut self, nnn: u16) {
        self.address_register = nnn;
    }
    pub fn opcode_jmp_nnn_plus_v0(&mut self, nnn: u16) {
        self.pc = nnn + (self.v[0] as u16);
        self.skip_increment_pc = true;
    }
    pub fn opcode_set_vx_random(&mut self, x: usize, nn: u8) {
        let random_number: u8 = self.rng.gen_range(0..=255);
        self.v[x] = random_number & nn;
    }

    // Draws a sprite at coordinate (VX,VY) that has a width of 8 pixels and a height of N pixels.
    // Each row of 8 pixels is read as bit-coded starting form memory location I.
    // I value doesn't change after the execution of this instruction
    // VF is set to 1 if any screen pixels are flipped from set to unset when the sprite is drawn,
    // and to 0 if that doesn't happen.
    // All drawing is XOR drawing (i.e. it toggles the screen pixels)
    // 0xDXYN
    pub fn opcode_draw(&mut self, x: usize, y: usize, n: u8) {
        self.v[0xF] = if self.draw_sprite(self.v[x], self.v[y], self.address_register, n) { 1 } else { 0 };
    }


    /**
    Display n-byte sprite starting at memory `address` at (x, y).
    Returns true if there's a collision.

    Eg.:
    Assuming the following sprite in memory at address 0x21A:

       Addr   Byte     Bits    Pixels
       0x21A  0xF0   11110000  ****
       0x21B  0x90   10010000  *  *
       0x21C  0x90   10010000  *  *
       0x21D  0x90   10010000  *  *
       0x21E  0xF0   11110000  ****

    Calling:

       self.draw_sprite(2, 3, 0x21A, 5)

    Will draw a big 0 on the display at (2, 3).
     */
    pub fn draw_sprite(&mut self, x: u8, y: u8, address: u16, height: u8) -> bool {
        let mut collision = false;

        let mut y_line = 0;
        while y_line < height {
            let pixel_memory_address = (address + y_line as u16) as usize;
            let mut pixel = self.memory[pixel_memory_address];
            let mut x_line: i8 = 7;
            while x_line >= 0 {
                if (pixel & 1) == 1 {
                    if self.display.draw(x + x_line as u8, y + y_line) {
                        collision = true;
                    }
                }
                pixel >>= 1;
                x_line -= 1;
            }
            y_line += 1;
        }

        return collision;
    }
    pub fn opcode_skip_key_pressed_in_vx(&mut self, x: usize) {
        if self.keypad.status(self.v[x] as usize) == 1 {
            self.increment_pc();
            self.skip_increment_pc = true;
        }
    }
    pub fn opcode_skip_key_not_pressed_in_vx(&mut self, x: usize) {
        if self.keypad.status(self.v[x] as usize) == 0 {
            self.increment_pc();
            self.skip_increment_pc = true;
        }
    }
    pub fn opcode_save_delay_to_vx(&mut self, x: usize) {
        self.v[x] = self.timer_delay;
    }
    pub fn opcode_wait_key(&mut self, x: usize) {
        let mut keypress = false;
        for i in 0..=V_SIZE {
            if self.keypad.status(i) != 0 {
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
            self.v[i] = (self.address_register + i as u16) as u8;
            i += 1;
        }
        self.address_register += x as u16 + 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_chip8() -> Chip8 {
        Chip8::new()
    }

    #[test]
    fn test_0x3xnn_should_skip_next_instruction_when_vx_equals_nn() {
        let mut emu = a_chip8();
        emu.pc = 0x200;
        emu.v[0] = 1;
        let nn = 1;

        emu.opcode_skip_if_vx_equals_nn(0, nn);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_0x3xnn_should_not_skip_instruction_when_vx_differs_nn() {
        let mut emu = a_chip8();
        emu.pc = 0x200;
        emu.v[0] = 0;
        let nn = 1;

        emu.opcode_skip_if_vx_equals_nn(0, nn);
        assert_eq!(emu.pc, 0x200);
    }

    #[test]
    fn test_0x4xnn_should_skip_next_instruction_if_vx_differs_nn() {
        let mut emu = a_chip8();
        emu.pc = 0x200;
        emu.v[0] = 0;
        let nn = 1;

        emu.opcode_skip_if_vx_diffs_nn(0, nn);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_0x4xnn_should_not_skip_next_instruction_if_vx_equals_nn() {
        let mut emu = a_chip8();
        emu.pc = 0x200;
        emu.v[0] = 1;
        let nn = 1;

        emu.opcode_skip_if_vx_diffs_nn(0, nn);
        assert_eq!(emu.pc, 0x200);
    }

    #[test]
    fn test_0x5xy0_should_skip_next_instruction_if_vx_equals_vy() {
        let mut emu = a_chip8();
        emu.pc = 0x200;
        emu.v[0] = 1;
        emu.v[1] = 1;

        emu.opcode_skip_if_vx_equals_vy(0, 1);
        assert_eq!(emu.pc, 0x202);
    }

    #[test]
    fn test_0x5xy0_should_not_skip_next_instruction_if_vx_differs_vy() {
        let mut emu = a_chip8();
        emu.pc = 0x200;
        emu.v[0] = 0;
        emu.v[1] = 1;

        emu.opcode_skip_if_vx_equals_vy(0, 1);
        assert_eq!(emu.pc, 0x200);
    }

    #[test]
    fn test_0x6xnn_should_put_nn_into_vx() {
        let mut emu = a_chip8();
        emu.pc = 0x200;
        emu.v[0] = 0;

        emu.opcode_set_vx_to_nn(0, 1);
        assert_eq!(emu.v[0], 1);
    }

    #[test]
    fn test_7xnn_should_add_nn_to_value_vx_and_store_in_vx() {
        let mut emu = a_chip8();
        emu.v[0] = 2;
        emu.opcode_adds_nn_to_vx(0, 4);

        assert_eq!(emu.v[0], 6);
    }

    #[test]
    fn test_7xnn_should_add_nn_to_value_vx_and_store_in_vx_with_wraparound() {
        let mut emu = a_chip8();
        emu.v[0] = 255;
        emu.opcode_adds_nn_to_vx(0, 1);

        assert_eq!(emu.v[0], 0);
    }

    #[test]
    fn test_8xy0_should_copy_vy_into_vx() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[1] = 20;
        emu.opcode_set_vx_to_vy(0x8000, 0, 1);

        assert_eq!(emu.v[0], 20);
    }

    #[test]
    fn test_8xy1_should_perform_bitwise_or_on_vx_and_vy_and_store_result_in_vx() {
        let mut emu = a_chip8();
        emu.v[0] = 0b01;
        emu.v[1] = 0b10;
        emu.opcode_set_vx_to_vy(0x8001, 0, 1);

        assert_eq!(emu.v[0], 0b11);
    }

    #[test]
    fn test_8xy2_should_perform_bitwise_and_on_vx_and_vy_and_store_result_in_vx() {
        let mut emu = a_chip8();
        emu.v[0] = 0b101;
        emu.v[1] = 0b100;
        emu.opcode_set_vx_to_vy(0x8002, 0, 1);

        assert_eq!(emu.v[0], 0b100);
    }

    #[test]
    fn test_8xy3_should_perform_bitwise_xor_on_vx_and_vy_and_store_result_in_vx() {
        let mut emu = a_chip8();
        emu.v[0] = 0b01010;
        emu.v[1] = 0b01001;
        emu.opcode_set_vx_to_vy(0x8003, 0, 1);

        assert_eq!(emu.v[0], 0b11);
    }

    #[test]
    fn test_8xy4_should_perform_add_on_vx_and_vy_and_store_result_in_vx_without_carry() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[1] = 5;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x8004, 0, 1);

        assert_eq!(emu.v[0], 15);
        assert_eq!(emu.v[1], 5, "Vy should keep original value.");
        assert_eq!(emu.v[0xF], 0, "No carry should have occurred");
    }

    #[test]
    fn test_8xy4_should_perform_add_on_vx_and_vy_and_store_result_in_vx_with_carry() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[1] = 255;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x8004, 0, 1);

        assert_eq!(emu.v[0], 9);
        assert_eq!(emu.v[1], 255, "Vy should keep original value.");
        assert_eq!(emu.v[0xF], 1, "Carry should have occurred");
    }

    #[test]
    fn test_8xy5_should_perform_subtract_on_vx_and_vy_and_store_result_in_vx_without_borrow() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[1] = 6;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x8005, 0, 1);

        assert_eq!(emu.v[0], 4);
        assert_eq!(emu.v[1], 6, "Vy should keep original value.");
        assert_eq!(emu.v[0xF], 1, "No borrow should have occurred");
    }

    #[test]
    fn test_8xy5_should_perform_subtract_on_vx_and_vy_and_store_result_in_vx_with_borrow() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[1] = 11;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x8005, 0, 1);

        assert_eq!(emu.v[0], 255);
        assert_eq!(emu.v[1], 11, "Vy should keep original value.");
        assert_eq!(emu.v[0xF], 0, "Borrow should have occurred");
    }

    #[test]
    fn test_8xy5_should_perform_subtract_on_vx_and_vy_and_store_result_in_zero_without_borrow() {
        let mut emu = a_chip8();
        emu.v[0] = 11;
        emu.v[1] = 11;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x8005, 0, 1);

        assert_eq!(emu.v[0], 0);
        assert_eq!(emu.v[1], 11, "Vy should keep original value.");
        assert_eq!(emu.v[0xF], 1, "No borrow should have occurred");
    }

    #[test]
    fn test_8xy6_should_set_vf_to_1_when_least_significant_bit_is_1_and_divide_by_2() {
        let mut emu = a_chip8();
        emu.v[0] = 11;
        emu.v[0xF] = 1;
        emu.opcode_set_vx_to_vy(0x8006, 0, 1);

        assert_eq!(emu.v[0], 11 >> 1);
        assert_eq!(emu.v[0xF], 1, "Flag should be enabled");
    }

    #[test]
    fn test_8xy6_should_not_set_vf_to_1_when_least_significant_bit_is_1_and_divide_by_2() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[0xF] = 1;
        emu.opcode_set_vx_to_vy(0x8006, 0, 1);

        assert_eq!(emu.v[0], 11 >> 1);
        assert_eq!(emu.v[0xF], 0, "Flag should be disabled");
    }

    #[test]
    fn test_8xy7_should_substract_vy_minus_vx_and_flag_borrow() {
        let mut emu = a_chip8();
        emu.v[0] = 6;
        emu.v[1] = 10;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x8007, 0, 1);

        assert_eq!(emu.v[0], 4);
        assert_eq!(emu.v[1], 10);
        assert_eq!(emu.v[0xF], 1, "Flag should be disabled");
    }

    #[test]
    fn test_8xy7_should_substract_vy_minus_vx_and_unflag_borrow() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[1] = 6;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x8007, 0, 1);

        assert_eq!(emu.v[0], 252);
        assert_eq!(emu.v[1], 6);
        assert_eq!(emu.v[0xF], 0, "Flag should be disabled");
    }

    #[test]
    fn test_8xy7_should_substract_vy_minus_vx_result_0_and_flag_borrow() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[1] = 10;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x8007, 0, 1);

        assert_eq!(emu.v[0], 0);
        assert_eq!(emu.v[1], 10);
        assert_eq!(emu.v[0xF], 1, "Flag should be disabled");
    }

    #[test]
    fn test_8xye_should_set_vf_to_1_when_most_significant_bit_of_vx_is_1_and_multiply_by_2() {
        let mut emu = a_chip8();
        emu.v[0] = 0b10000001;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x800E, 0, 0);

        assert_eq!(emu.v[0], 2);
        assert_eq!(emu.v[0xF], 1, "Flag should be enabled");
    }

    #[test]
    fn test_8xye_should_not_set_vf_to_1_when_most_significant_bit_of_vx_is_0_and_multiply_by_2() {
        let mut emu = a_chip8();
        emu.v[0] = 0b00000001;
        emu.v[0xF] = 0;
        emu.opcode_set_vx_to_vy(0x800E, 0, 0);

        assert_eq!(emu.v[0], 2);
        assert_eq!(emu.v[0xF], 0, "Flag should be enabled");
    }

    #[test]
    fn test_9xy0_should_skip_next_instruction_if_vx_differs_from_vy() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[1] = 9;
        emu.pc = 0x200;
        emu.opcode_skips_if_vx_diffs_vy(0, 1);

        assert_eq!(emu.pc, 0x202);
        assert_eq!(emu.skip_increment_pc, true);
    }

    #[test]
    fn test_9xy0_should_not_skip_next_instruction_if_vx_equals_vy() {
        let mut emu = a_chip8();
        emu.v[0] = 10;
        emu.v[1] = 10;
        emu.pc = 0x200;
        emu.opcode_skips_if_vx_diffs_vy(0, 1);

        assert_eq!(emu.pc, 0x200);
        assert_eq!(emu.skip_increment_pc, false);
    }

    #[test]
    fn test_annn_should_set_i_to_nnn() {
        let mut emu = a_chip8();
        emu.address_register = 0;

        emu.opcode_set_i_to_nnn(0xFFF);

        assert_eq!(emu.address_register, 0xFFF);
    }

    #[test]
    fn test_bnnn_jumps_to_location() {
        let mut emu = a_chip8();
        emu.pc = 0x200;
        emu.v[0] = 0xA;

        emu.opcode_jmp_nnn_plus_v0(0x400);

        assert_eq!(emu.pc, 0x40A);
        assert_eq!(emu.skip_increment_pc, true);
    }

    #[test]
    #[ignore]
    fn test_cnnn_generate_random_value() {
        let mut emu = a_chip8();

        // Implement by mocking random generator.
    }

    #[test]
    #[ignore]
    fn test_dnnn_displays_sprite() {
        let mut emu = a_chip8();
        emu.pc = 0x200;
        emu.v[0] = 0xA;

        emu.opcode_jmp_nnn_plus_v0(0x400);

        assert_eq!(emu.pc, 0x40A);
        assert_eq!(emu.skip_increment_pc, true);
    }

    #[test]
    fn test_ex9e_should_skip_if_key_is_pressed() {
        let mut emu  = a_chip8();
        let key_index = 1;
        emu.v[0] = key_index;
        emu.pc = 0x200;
        emu.keypad.press(key_index as usize);

        emu.opcode_skip_key_pressed_in_vx(0);

        assert_eq!(emu.pc, 0x202);
        assert_eq!(emu.skip_increment_pc, true);
    }

    #[test]
    fn test_ex9e_should_not_skip_if_key_is_not_pressed() {
        let mut emu  = a_chip8();
        let key_index = 1;
        emu.v[0] = key_index;
        emu.pc = 0x200;
        emu.keypad.release(key_index as usize);

        emu.opcode_skip_key_pressed_in_vx(0);

        assert_eq!(emu.pc, 0x200);
        assert_eq!(emu.skip_increment_pc, false);
    }
    #[test]
    fn test_exa1_should_skip_if_key_is_not_pressed() {
        let mut emu  = a_chip8();
        let key_index = 1;
        emu.v[0] = key_index;
        emu.pc = 0x200;
        emu.keypad.release(key_index as usize);

        emu.opcode_skip_key_not_pressed_in_vx(0);

        assert_eq!(emu.pc, 0x202);
        assert_eq!(emu.skip_increment_pc, true);
    }

    #[test]
    fn test_exa1_should_not_skip_if_key_is_pressed() {
        let mut emu  = a_chip8();
        let key_index = 1;
        emu.v[0] = key_index;
        emu.pc = 0x200;
        emu.keypad.press(key_index as usize);

        emu.opcode_skip_key_not_pressed_in_vx(0);

        assert_eq!(emu.pc, 0x200);
        assert_eq!(emu.skip_increment_pc, false);
    }
}