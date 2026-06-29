pub enum FlippedOffResult {
    Yes,
    No,
}

pub trait Display {
    fn draw(x: u8, y: u8) -> FlippedOffResult;
}

const MEMORY_SIZE: usize = 4096;
const STACK_SIZE: usize = 12;
const REGISTER_SIZE: usize = 16;

pub struct Emulator {
    memory: [u8; MEMORY_SIZE],
    program_counter: usize,
    stack: [u16; STACK_SIZE],
    stack_counter: usize,
    registers: [u8; REGISTER_SIZE],
}
pub trait KeyboardReader {
    fn is_key_pressed(key: u8) -> bool;

    /// if multiple keys are pressed, will return any of these
    fn get_pressed_key() -> u8;
}
