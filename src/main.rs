use piston_window::{WindowSettings, PistonWindow};

fn main() {
    let mut window: PistonWindow = WindowSettings::new(
        "Tetris", [300, 800]).exit_on_esc(true).build().unwrap();

    while let Some(e) = window.next() {
        println!("{:?}", e);
    }
}
