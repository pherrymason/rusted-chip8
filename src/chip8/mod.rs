use rand::rngs::ThreadRng;

mod opcodes;

struct Keypad {}

impl Keypad {
    pub fn status(&self, key: u8) -> u8 {
        return 0;
    }
}

struct Display {}

const STACK_SIZE: usize = 200;
const MEMORY_SIZE: usize = 4096;
const V_SIZE: usize = 16;
const PROGRAM_START_LOCATION: usize = 0x200;
const PROGRAM_SIZE: usize = MEMORY_SIZE - PROGRAM_START_LOCATION;

pub struct Chip8 {
    keypad: Keypad,
    display: Display,
    memory: Vec<u8>,
    v: [u8; V_SIZE],
    address_register: u16,
    pc: u16,
    stack: Stack,
    stack_pointer: u8,

    play: bool,
    clear_screen: bool,

    timer_delay: u8,
    timer_sound: u8,
    rng: ThreadRng
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            keypad: Keypad {},
            display: Display {},
            memory: [0; MEMORY_SIZE],
            memory: vec![0; PROGRAM_START_LOCATION],
            v: [0; 16],
            address_register: 0,
            pc: 0,
            stack: Stack::new(),
            stack_pointer: 0,
            play: false,
            clear_screen: false,
            timer_delay: 0,
            timer_sound: 0,
            rng: rand::thread_rng()
        }
    }

    pub fn reset(&mut self) {
        self.memory = vec![0; 80];
        self.v = [0; 16];
        self.address_register = 0;
        self.pc = PROGRAM_START_LOCATION as u16;
        self.stack = Stack::new();
        self.stack_pointer = 0;
        // TODO reset keypad
        // TODO reset something_left_to_draw flag
        // TODO reset clear_screen flag.
        self.load_font();
    }

    fn load_font(&mut self) {
        /*
        this.fontSet = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];

    // Memory map
    // 0x000 - 0x1FF	Chip 8 interpreter (contains font set in emu)
    // 0x050 - 0x0A0	Used for the built in 4x5 pixel font set (0-F)
    // 0x200 - 0xFFF	Program ROM and work RAM
    for (var i=0;i<80; i++){
        this.memory[0x50+i] = this.fontSet[i];
    }
         */
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.reset();
        self.memory.extend_from_slice(&program);
    }

    pub fn start(&mut self) {
        self.play = true;
    }

    pub fn stop(&mut self) {
        self.play = false;
    }

    pub fn pause(&mut self) {
        self.play = false;
    }

    fn tick(&mut self) {
        let opcode: u16 = ((self.memory[self.pc as usize] as u16) << 8) + self.memory[(self.pc as usize) + 1] as u16;
        self.execute_operation(opcode);
    }

    fn execute_operation(&mut self, opcode: u16) {
        let X: u8 = ((opcode & 0x0F00) >> 8) as u8;
        let Y: u8 = ((opcode & 0x00F0) >> 4) as u8;
        let N: u8 = (opcode & 0x000F) as u8;
        let NN: u8 = (opcode & 0x00FF) as u8;
        let NNN = opcode & 0x0FFF;

        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00E0 => self.opcode_clear_screen(),
                    0x00EE => self.return_from_subroutine(),
                    0x00FF => self.set_schip_graphic_mode(),
                    _ => panic!("unknown opcode")
                }
            },
            0x1000 => self.opcode_jmp(NNN),
            0x2000 => self.opcode_call_subroutine(NNN),
            0x3000 => self.opcode_skip_if_vx_equals_nn(X, NN),
            0x4000 => self.opcode_skip_if_vx_diffs_nn(X, NN),
            0x5000 => self.opcode_skip_if_vx_equals_vy(X, Y),
            0x6000 => self.opcode_set_vx_to_nn(X, NN),
            0x7000 => self.opcode_adds_nn_to_vx(X,NN),
            0x8000 => self.opcode_set_vx_to_vy(opcode,X as usize,Y as usize),
            0x9000 => self.opcode_skips_if_vx_diffs_vy(X as usize,Y as usize),
            0xA000 => self.opcode_set_i_to_nnn(NNN),
            0xB000 => self.opcode_jmp_nnn_plus_v0(NNN),
            0xC000 => self.opcode_set_vx_random(X as usize, NN),
            0xD000 => self.opcode_draw(X as usize, Y as usize, N),
            0xE000 => {
                match opcode&0xFF {
                    0x9E => self.opcode_skip_key_pressed_in_vx(X as usize),
                    0xA1 => self.opcode_skip_key_not_pressed_in_vx(X as usize),
                    _ => panic!("Unhandled opcode")
                }
            }
            0xF000 => {
                match opcode &0xFF {
                    0x07 => self.opcode_save_delay_to_vx(X as usize),
                    0x0A => self.opcode_wait_key(X as usize),
                    0x15 => self.opcode_save_vx_to_delay(X as usize),
                    0x18 => self.opcode_save_vx_to_sound_timer(X as usize),
                    0x1E => self.opcode_adds_vx_to_i(X as usize),
                    0x29 => self.opcode_set_i_with_vx(X as usize),
                    0x33 => self.opcode_save_bin_vx(X as usize),
                    0x55 => self.opcode_dump_v_to_memory(X as usize),
                    0x65 => self.opcode_fill_v_with_memory(X as usize),
                    _ => panic!("Unhandled opcode")
                }
            }
            _ => println!("Unknown Opcode {}", opcode),
        }
        self.increment_pc();
    }

    fn increment_pc(&mut self) {
        self.pc += 1;
    }
}

struct Stack {
    data: [u16; STACK_SIZE],
    top: usize,
}

impl Stack {
    pub fn new() -> Self {
        Stack {
            data: [0; STACK_SIZE],
            top: 0,
        }
    }

    pub fn push(&mut self, value: u16) {
        if self.top < STACK_SIZE {
            self.data[self.top] = value;
            self.top += 1
        }
    }

    pub fn pop(&mut self) -> u16 {
        if self.top > 0 {
            self.top -= 1;
            return self.data[self.top];
        }

        return 0;
    }
}