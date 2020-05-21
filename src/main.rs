use piston_window::{WindowSettings, PistonWindow, Event, RenderEvent, PressEvent};
use piston_window::{Rectangle, DrawState, Context, Graphics};
use piston_window::{Button, Key};

use rand::Rng;

use std::time::{Duration, Instant};
use std::collections::HashMap;

#[derive(Copy, Clone)]
enum Color {
    Red, Green,
}

#[derive(Default, Clone)]
struct Board(HashMap<(i8, i8), Color>);

impl Board {
    fn new(v: &[(i8, i8)], color: Color) -> Self {
        Board(v.iter().cloned().map(|(x, y)| ((x, y), color)).collect())
    }

    fn modified<F>(&self, f: F) -> Self
        where F: Fn((i8, i8)) -> (i8, i8)
    {
        Board(self.0.iter().map(|((x, y), color)| (f((*x, *y)), *color)).collect())
    }

    fn transposed(&self) -> Self {
        self.modified(|(ox, oy)| (oy, ox))
    }

    fn mirrored_y(&self) -> Self {
        self.modified(|(ox, oy)| (ox, -oy))
    }

    fn rotated(&self) -> Self {
        self.mirrored_y().transposed()
    }

    fn rotated_counter(&self) -> Self {
        self.rotated().rotated().rotated()
    }

    fn negative_shift(&self) -> (i8, i8) {
        use std::cmp::min;

        self.0.keys().into_iter().cloned()
            .fold((0, 0), |(mx, my), (ox, oy)| (min(mx, ox), min(my, oy)))
    }

    fn shifted(&self, (x, y): (i8, i8)) -> Self {
        self.modified(|(ox, oy)| (ox + x, oy + y))
    }

    fn merged(&self, other: &Board) -> Option<Self> {
        let mut hashmap = HashMap::new();
        hashmap.extend(other.0.iter());
        hashmap.extend(self.0.iter());

        if hashmap.len() != self.0.len() + other.0.len() {
            return None;
        }

        Some(Self(hashmap))
    }

    fn contained(&self, x: i8, y: i8) -> bool {
        self.0.keys().into_iter().cloned()
            .fold(true, |b, (ox, oy)| b && ox < x && oy < y && ox >= 0 && oy >= 0)
    }

    fn render<G>(
        &self,
        metrics: &Metrics,
        c: &Context,
        g: &mut G,
    )
        where G: Graphics
    {
        let mut draw = |color, rect: [f64; 4]| {
            Rectangle::new(color).draw(rect, &DrawState::default(), c.transform, g);
        };

        for x in 0 .. metrics.board_x {
            for y in 0 .. metrics.board_y {
                let block_pixels = metrics.block_pixels as f64;
                let border_size = block_pixels / 20.0;
                let outer = [block_pixels * (x as f64), block_pixels * (y as f64), block_pixels, block_pixels];
                let inner = [outer[0] + border_size, outer[1] + border_size,
                outer[2] - border_size * 2.0, outer[3] - border_size * 2.0];

                draw([0.2, 0.2, 0.2, 1.0], outer);
                draw([0.1, 0.1, 0.1, 1.0], inner);

                if let Some(color) = self.0.get(&(x as i8, y as i8)) {
                    let code = match color {
                        Color::Red     => [1.0, 0.0, 0.0, 1.0],
                        Color::Green   => [0.0, 1.0, 0.0, 1.0],
                    };
                    draw(code, outer);
                    let code = [code[0]*0.8, code[1]*0.8, code[2]*0.8, code[3]];
                    draw(code, inner);
                }
            }
        }
    }
}

#[derive(Default)]
struct Metrics {
    block_pixels: usize,
    board_x: usize,
    board_y: usize,
}

impl Metrics {
    fn resolution(&self) -> [u32; 2] {
        [(self.board_x * self.block_pixels) as u32,
         (self.board_y * self.block_pixels) as u32]
    }
}

struct Game {
    board: Board,
    metrics: Metrics,
    falling: Board,
    shift: (i8, i8),
    possible_pieces: Vec<Board>,
    time_since_fall: Instant,
}

impl Game {
    fn new(metrics: Metrics) -> Self {
        Self {
            metrics,
            board: Default::default(),
            falling: Default::default(),
            time_since_fall: Instant::now(),
            shift: (0, 0),
            possible_pieces: vec![
                Board::new(&[
                    (0, 0),
                    (0, 1),
                    (1, 0),
                    (1, 1),
                ][..], Color::Red),
                Board::new(&[
                    (0, 0),
                    (1, 0),
                    (1, 1),
                    (2, 0),
                ][..], Color::Green),
            ]
        }
    }

    fn new_falling(&mut self) {
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0, self.possible_pieces.len());

        self.falling = self.possible_pieces[idx].clone();
        self.shift = (0, 0);
    }

    fn render(&self, window: &mut PistonWindow, event: &Event) {
        let merged = self.board.merged(&self.falling_shifted()).unwrap();

        window.draw_2d(event, |c, g, _| {
            merged.render(&self.metrics, &c, g);
        });
    }

    fn falling_shifted(&self) -> Board {
        self.falling.shifted(self.shift)
    }

    fn progress(&mut self) {
        if self.time_since_fall.elapsed() <= Duration::from_millis(700) {
            return;
        }

        self.move_falling(0, 1);
        self.time_since_fall = Instant::now();
    }

    fn move_falling(&mut self, x: i8, y: i8) {
        let falling = self.falling_shifted().shifted((x, y));
        let merged = self.board.merged(&falling);
        let contained = falling.contained(self.metrics.board_x as i8,
                                          self.metrics.board_y as i8);

        if merged.is_some() && contained {
            // Allow the movement
            self.shift.0 += x;
            self.shift.1 += y;
            return
        }

        if let (0, 1) = (x, y) {
            self.board = self.board.merged(&self.falling_shifted()).unwrap();
            self.new_falling();
        }
    }

    fn on_press(&mut self, args: &Button) {
        match args {
            Button::Keyboard(key) => { self.on_key(key); }
            _ => {},
        }
    }

    fn on_key(&mut self, key: &Key) {
        let movement = match key {
            Key::Right => Some((1, 0)),
            Key::Left => Some((-1, 0)),
            Key::Down => Some((0, 1)),
            _ => None,
        };

        if let Some(movement) = movement {
            self.move_falling(movement.0, movement.1);
            return;
        }

        match key {
            Key::Up => self.rotate(false),
            Key::NumPad5 => self.rotate(true),
            _ => return,
        }
    }

    fn rotate(&mut self, counter: bool) {
        let rotated = if counter {
            self.falling.rotated()
        } else {
            self.falling.rotated_counter()
        };
        let (x, y) = rotated.negative_shift();
        let falling = rotated.shifted((-x, -y));

        if let Some(merged) = self.board.merged(&falling.shifted(self.shift)) {
            if merged.contained(self.metrics.board_x as i8,
                                self.metrics.board_y as i8)
            {
                // Allow the rotation
                self.falling = falling;
                return
            }
        }
    }
}

fn main() {
    let metrics = Metrics {
        block_pixels: 20,
        board_x: 8,
        board_y: 20,
    };

    let mut window: PistonWindow = WindowSettings::new(
        "Tetris", metrics.resolution()).exit_on_esc(true).build().unwrap();
    let mut game = Game::new(metrics);

    game.new_falling();

    while let Some(e) = window.next() {
        game.progress();

        if let Some(_) = e.render_args() {
            game.render(&mut window, &e);
        }

        if let Some(args) = e.press_args() {
            game.on_press(&args);
        }
    }
}
