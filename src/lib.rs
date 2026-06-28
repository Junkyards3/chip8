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

