use std::time::{Duration, Instant};

pub enum FlippedOffResult {
    Yes,
    No,
}

pub trait Display {
    fn draw(&mut self, x: u8, y: u8, draw: bool) -> FlippedOffResult;

    fn clear(&mut self);
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
    index_register: u16,
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
            index_register: 0,
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
        self.program_counter += 2;
        match Instruction::from_opcode(opcode) {
            Instruction::ClearScreen => self.display.clear(),
            Instruction::Jump => self.program_counter = get_nnn(opcode),
            Instruction::SetRegisterConstant => {
                self.registers[get_x(opcode) as usize] = get_nn(opcode)
            }
            Instruction::AddRegisterConstant => {
                let _ = self.registers[get_x(opcode) as usize].overflowing_add(get_nn(opcode));
            }
            Instruction::SetIndexRegister => self.index_register = get_nnn(opcode),
            Instruction::Draw => {
                let x = self.registers[get_x(opcode) as usize] & 63;
                let y = self.registers[get_y(opcode) as usize] & 31;
                self.registers[0xf] = 0;
                let n = get_n(opcode);
                for r in 0..n {
                    let sprite_byte = self.memory[self.index_register as usize];
                }
            }
            _ => todo!(),
        }
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

enum Instruction {
    ClearScreen,
    Jump,
    JumpOffset,
    SetRegisterConstant,
    AddRegisterConstant,
    CopyBetweenRegister,
    SetBetweenRegister,
    SetOrRegister,
    SetAndRegister,
    SetXorRegister,
    AddBetweenRegister,
    SubBetweenRegister,
    OppSubBetweenRegister,
    ShiftRightBetweenRegister,
    ShiftLeftBetweenRegister,
    SetIndexRegister,
    Draw,
    Return,
    Subroutine,
    SkipEqualConstant,
    SkipUnequalConstant,
    SkipEqualRegister,
    SkipUnequalRegister,
    MachineSubroutine,
    RandomNumber,
    SkipKeyHeld,
    SkipKeyNotHeld,
    SetRegisterFromDelayTimer,
    WaitKeyPress,
    SetDelayTimerFromRegister,
    SetSoundTimerFromRegister,
    AddIndexRegister,
    SetIndexSpriteData,
    StoreBCD,
    StoreAllRegisters,
    FillAllRegisters,
}

impl Instruction {
    fn from_opcode(opcode: u16) -> Self {
        match opcode << 12 {
            0x0 => Self::from_opcode_0(opcode),
            0x1 => Self::Jump,
            0x2 => Self::Subroutine,
            0x3 => Self::SkipEqualConstant,
            0x4 => Self::SkipUnequalConstant,
            0x5 => Self::SkipEqualRegister,
            0x6 => Self::SetRegisterConstant,
            0x7 => Self::AddRegisterConstant,
            0x8 => Self::from_opcode_8(opcode),
            0x9 => Self::SkipUnequalRegister,
            0xa => Self::SetIndexRegister,
            0xb => Self::JumpOffset,
            0xc => Self::RandomNumber,
            0xd => Self::Draw,
            0xe => Self::from_opcode_e(opcode),
            0xf => Self::from_opcode_f(opcode),
            _ => panic!("unknown instruction"),
        }
    }

    fn from_opcode_0(opcode: u16) -> Self {
        match opcode & 0xff {
            0xe0 => Self::ClearScreen,
            0xee => Self::Return,
            _ => Self::MachineSubroutine,
        }
    }

    fn from_opcode_8(opcode: u16) -> Instruction {
        match opcode & 0xf {
            0x0 => Self::SetBetweenRegister,
            0x1 => Self::SetOrRegister,
            0x2 => Self::SetAndRegister,
            0x3 => Self::SetXorRegister,
            0x4 => Self::AddBetweenRegister,
            0x5 => Self::SubBetweenRegister,
            0x6 => Self::ShiftRightBetweenRegister,
            0x7 => Self::OppSubBetweenRegister,
            0xe => Self::ShiftLeftBetweenRegister,
            _ => panic!("unknown instruction"),
        }
    }

    fn from_opcode_e(opcode: u16) -> Instruction {
        match opcode & 0xff {
            0x9e => Self::SkipKeyHeld,
            0xa1 => Self::SkipKeyNotHeld,
            _ => panic!("unknown instruction"),
        }
    }

    fn from_opcode_f(opcode: u16) -> Instruction {
        match opcode & 0xff {
            0x07 => Self::SetRegisterFromDelayTimer,
            0x0a => Self::WaitKeyPress,
            0x15 => Self::SetDelayTimerFromRegister,
            0x18 => Self::SetSoundTimerFromRegister,
            0x1e => Self::AddIndexRegister,
            0x29 => Self::SetIndexSpriteData,
            0x33 => Self::StoreBCD,
            0x55 => Self::StoreAllRegisters,
            0x65 => Self::FillAllRegisters,
            _ => panic!("unknown instruction"),
        }
    }
}
