use std::time::{Duration, Instant};

pub enum FlippedOffResult {
    Yes,
    No,
}

pub trait Display {
    fn draw(x: u8, y: u8) -> FlippedOffResult;
}

pub trait KeyboardReader {
    fn is_key_pressed(key: u8) -> bool;

    /// if multiple keys are pressed, will return any of these
    fn get_pressed_key() -> u8;
}

const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 12;
const REGISTER_SIZE: usize = 16;

const CPU_HZ: u32 = 700;
const TIMER_HZ: u32 = 60;

const CPU_STEP: Duration = Duration::from_nanos(1_000_000_000 / CPU_HZ as u64);
const TIMER_STEP: Duration = Duration::from_nanos(1_000_000_000 / TIMER_HZ as u64);

const FONT_ADDRESS: usize = 0x50;
const FONT_MEMORY: [u8; 5 * 16] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

const PROGRAM_ADDRESS: usize = 0x200;

pub struct Emulator<D: Display, KR: KeyboardReader> {
    memory: [u8; MEMORY_SIZE],
    program_counter: u16,
    stack: [u16; STACK_SIZE],
    stack_counter: usize,
    registers: [u8; REGISTER_SIZE],
    display: D,
    keyboard_reader: KR,
    delay_timer: u8,
    sound_timer: u8,
    draw_flag: bool,
}

impl<D: Display, KR: KeyboardReader> Emulator<D, KR> {
    pub fn init(display: D, keyboard_reader: KR, program: &[u8]) -> Self {
        let mut memory = [0u8; MEMORY_SIZE];
        memory[FONT_ADDRESS..FONT_ADDRESS + FONT_MEMORY.len()].copy_from_slice(&FONT_MEMORY);
        memory[PROGRAM_ADDRESS..PROGRAM_ADDRESS + program.len()].copy_from_slice(program);

        Emulator {
            memory,
            program_counter: PROGRAM_ADDRESS as u16,
            stack: [0u16; STACK_SIZE],
            stack_counter: 0,
            registers: [0u8; REGISTER_SIZE],
            display,
            keyboard_reader,
            delay_timer: 0,
            sound_timer: 0,
            draw_flag: false,
        }
    }

    fn step(&mut self) {
        let opcode = ((self.memory[self.program_counter as usize] as u16) << 8)
            | (self.memory[(self.program_counter + 1) as usize] as u16);
    }

    pub fn run(&mut self) {
        let mut last_cpu = Instant::now();
        let mut last_timer = Instant::now();

        loop {
            let now = Instant::now();

            while now.duration_since(last_cpu) >= CPU_STEP {
                self.step(); // exécute 1 instruction
                last_cpu += CPU_STEP;
            }

            while now.duration_since(last_timer) >= TIMER_STEP {
                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }

                if self.sound_timer > 0 {
                    self.sound_timer -= 1;
                    // jouer un bip si nécessaire
                }

                last_timer += TIMER_STEP;
            }

            if self.draw_flag {
                //render_screen();
                self.draw_flag = false;
            }

            //handle_input();

            std::thread::sleep(Duration::from_millis(1));
        }
    }

    pub fn run_full_speed(&mut self, instructions_count: usize) {
        let mut timer_accumulator = 0;
        for _ in 0..instructions_count {
            self.step();

            timer_accumulator += TIMER_HZ;

            while timer_accumulator >= CPU_HZ {
                timer_accumulator -= CPU_HZ;

                if self.delay_timer > 0 {
                    self.delay_timer -= 1;
                }

                if self.sound_timer > 0 {
                    self.sound_timer -= 1;
                }
            }
        }
    }
}

fn get_x(opcode: u16) -> u8 {
    ((opcode >> 8) & 0xF) as u8
}

fn get_y(opcode: u16) -> u8 {
    ((opcode >> 4) & 0xF) as u8
}

fn get_n(opcode: u16) -> u8 {
    (opcode & 0xF) as u8
}

fn get_nn(opcode: u16) -> u8 {
    (opcode & 0xFF) as u8
}

fn get_nnn(opcode: u16) -> u16 {
    opcode & 0xFFF
}
