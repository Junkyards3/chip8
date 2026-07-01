use std::time::{Duration, Instant};

pub enum FlippedOffResult {
    Yes,
    No,
}

const SCREEN_WIDTH: u8 = 64;
const SCREEN_HEIGHT: u8 = 32;

pub trait Display {
    fn switch(&mut self, x: u8, y: u8) -> FlippedOffResult;

    fn redraw(&mut self);

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
                self.registers[get_x(opcode) as usize] = self.registers[get_x(opcode) as usize]
                    .overflowing_add(get_nn(opcode))
                    .0;
            }
            Instruction::SetIndexRegister => self.index_register = get_nnn(opcode),
            Instruction::Draw => {
                let x = self.registers[get_x(opcode) as usize] & (SCREEN_WIDTH - 1);
                let y = self.registers[get_y(opcode) as usize] & (SCREEN_HEIGHT - 1);
                self.registers[0xf] = 0;
                let n = get_n(opcode);
                for iy in 0..n {
                    let sprite_byte = self.memory[(self.index_register + (iy as u16)) as usize];

                    for ix in 0..8 {
                        let should_switch = sprite_byte & (1 << (7 - ix)) != 0;
                        if should_switch
                            && matches!(self.display.switch(x + ix, y + iy), FlippedOffResult::Yes)
                        {
                            self.registers[0xf] = 1;
                        }
                        if x + ix == SCREEN_WIDTH - 1 {
                            break;
                        }
                    }

                    if y + iy == SCREEN_HEIGHT - 1 {
                        break;
                    }
                }

                self.draw_flag = true;
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
                self.display.redraw();
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

            if self.draw_flag {
                self.display.redraw();
                self.draw_flag = false;
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

#[derive(Debug)]
enum Instruction {
    ClearScreen,               //0NNN
    Jump,                      //1NNN
    JumpOffset,                //BNNN
    SetRegisterConstant,       //6XNN
    AddRegisterConstant,       //7XNN
    SetBetweenRegister,        //8XY0
    SetOrRegister,             //8XY1
    SetAndRegister,            //8XY2
    SetXorRegister,            //8XY3
    AddBetweenRegister,        //8XY4
    SubBetweenRegister,        //8XY5
    OppSubBetweenRegister,     //8XY7
    ShiftRightBetweenRegister, //8XY6
    ShiftLeftBetweenRegister,  //8XYE
    SetIndexRegister,          //ANNN
    Draw,                      //DXYN
    Return,                    //00EE
    Subroutine,                //2NNN
    SkipEqualConstant,         //3XNN
    SkipUnequalConstant,       //4XNN
    SkipEqualRegister,         //5XY0
    SkipUnequalRegister,       //9XY0
    MachineSubroutine,         //0NNN
    RandomNumber,              //CXNN
    SkipKeyHeld,               //EX9E
    SkipKeyNotHeld,            //EXA1
    SetRegisterFromDelayTimer, //FX07
    WaitKeyPress,              //FX0A
    SetDelayTimerFromRegister, //FX15
    SetSoundTimerFromRegister, //FX18
    AddIndexRegister,          //FX1E
    SetIndexSpriteData,        //FX29
    StoreBCD,                  //FX33
    StoreAllRegisters,         //FX55
    FillAllRegisters,          //FX65
}

impl Instruction {
    fn from_opcode(opcode: u16) -> Self {
        match opcode >> 12 {
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
            _ => panic!("unknown instruction {opcode:#x}"),
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
            _ => panic!("unknown instruction {opcode:#x}"),
        }
    }

    fn from_opcode_e(opcode: u16) -> Instruction {
        match opcode & 0xff {
            0x9e => Self::SkipKeyHeld,
            0xa1 => Self::SkipKeyNotHeld,
            _ => panic!("unknown instruction {opcode:#x}"),
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
            _ => panic!("unknown instruction {opcode:#x}"),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        cell::RefCell,
        io::{Write, stdout},
        rc::Rc,
    };

    use crate::{Display, Emulator, FlippedOffResult, KeyboardReader, SCREEN_HEIGHT, SCREEN_WIDTH};

    struct DummyDisplay {
        current: [[bool; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],
        buffer: [[bool; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize],
    }

    fn get_pretty(value: bool) -> &'static [u8] {
        match value {
            true => "██".as_bytes(),
            false => "  ".as_bytes(),
        }
    }

    fn get_direct(value: bool) -> &'static [u8] {
        match value {
            true => "#".as_bytes(),
            false => " ".as_bytes(),
        }
    }

    impl DummyDisplay {
        fn new() -> Self {
            let current = [[false; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize];
            let buffer = [[false; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize];
            DummyDisplay { current, buffer }
        }

        fn write<W: Write>(&self, mut writer: W, pretty: bool) -> std::io::Result<()> {
            for row in &self.current {
                for &pixel in row {
                    writer.write_all(if pretty {
                        get_pretty(pixel)
                    } else {
                        get_direct(pixel)
                    })?;
                }

                writer.write_all(b"\n")?;
            }

            Ok(())
        }

        fn write_pretty<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
            self.write(writer, true)
        }

        fn write_direct<W: Write>(&self, mut writer: W) -> std::io::Result<()> {
            self.write(writer, false)
        }
    }

    impl Display for DummyDisplay {
        fn switch(&mut self, x: u8, y: u8) -> crate::FlippedOffResult {
            self.buffer[y as usize][x as usize] = !self.buffer[y as usize][x as usize];
            match self.buffer[y as usize][x as usize] {
                true => FlippedOffResult::No,
                false => FlippedOffResult::Yes,
            }
        }

        fn redraw(&mut self) {
            self.current.copy_from_slice(&self.buffer);
        }

        fn clear(&mut self) {
            self.buffer = [[false; SCREEN_WIDTH as usize]; SCREEN_HEIGHT as usize];
        }
    }

    impl Display for Rc<RefCell<DummyDisplay>> {
        fn switch(&mut self, x: u8, y: u8) -> crate::FlippedOffResult {
            self.borrow_mut().switch(x, y)
        }

        fn redraw(&mut self) {
            self.borrow_mut().redraw()
        }

        fn clear(&mut self) {
            self.borrow_mut().clear()
        }
    }

    struct DummyKeyboard;

    impl KeyboardReader for DummyKeyboard {
        fn is_key_pressed(key: u8) -> bool {
            todo!()
        }

        fn get_pressed_key() -> u8 {
            todo!()
        }
    }

    #[test]
    fn load_ibm() {
        let program = std::fs::read("./programs/ibm-logo.ch8").unwrap();

        let display = Rc::new(RefCell::new(DummyDisplay::new()));
        let keyboard_reader = DummyKeyboard {};
        let mut emulator = Emulator::init(Rc::clone(&display), keyboard_reader, &program);

        emulator.run_full_speed(100);

        let expected = std::fs::read("./results/ibm-logo.txt").unwrap();
        let mut actual = Vec::new();
        display.borrow().write_direct(&mut actual).unwrap();

        assert_eq!(actual, expected, "\nActual screen:\n{}", {
            let mut pretty = Vec::new();
            display.borrow().write_pretty(&mut pretty).unwrap();
            String::from_utf8(pretty).unwrap()
        });
    }
}
