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

pub struct Emulator<D: Display, KR: KeyboardReader> {
    memory: [u8; MEMORY_SIZE],
    program_counter: usize,
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
    fn step(&self) {}

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

    pub fn run_full_speed(&mut self) {
        let mut timer_accumulator = 0;
        loop {
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
