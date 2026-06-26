enum FlippedOffResult {
    Yes,
    No,
}

trait Display {
    fn draw(x: u8, y: u8) -> FlippedOffResult;
}

trait KeyboardReader {
    fn is_key_pressed(key: u8) -> bool;

    /// if multiple keys are pressed, will return any of these
    fn get_pressed_key() -> u8;
}

fn main() {
    println!("Hello, world!");
}
