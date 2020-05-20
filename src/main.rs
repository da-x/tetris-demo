use piston_window::{WindowSettings, PistonWindow, Event, RenderEvent};
use piston_window::{Rectangle, DrawState, Context, Graphics};

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

    fn shifted(&self, (x, y): (i8, i8)) -> Self {
        self.modified(|(ox, oy)| (ox + x, oy + y))
    }

    fn merged(&self, other: &Board) -> Self {
        let mut hashmap = HashMap::new();
        hashmap.extend(other.0.iter());
        hashmap.extend(self.0.iter());
        Self(hashmap)
    }

    fn contained(&self, x: i8, y: i8) -> bool {
        self.0.keys().into_iter().cloned()
            .fold(true, |b, (ox, oy)| b && ox < x && oy < y && x >= 0 && y >= 0)
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
        let merged = self.board.merged(&self.falling_shifted());

        window.draw_2d(event, |c, g, _| {
            merged.render(&self.metrics, &c, g);
        });
    }

    fn falling_shifted(&self) -> Board {
        self.falling.shifted(self.shift)
    }

    fn progress(&mut self) {
        if self.time_since_fall.elapsed() <= Duration::from_millis(70) {
            return;
        }

        self.move_falling(0, 1);
        self.time_since_fall = Instant::now();
    }

    fn move_falling(&mut self, x: i8, y: i8) {
        let falling = self.falling_shifted().shifted((x, y));

        if falling.contained(self.metrics.board_x as i8,
                             self.metrics.board_y as i8)
        {
            self.shift.0 += x;
            self.shift.1 += y;
        } else {
            self.board = self.board.merged(&self.falling_shifted());
            self.new_falling();
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
    }
}
