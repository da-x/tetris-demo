use piston_window::{WindowSettings, PistonWindow, Event, RenderEvent};
use piston_window::{Rectangle, DrawState, Context, Graphics};

use std::collections::HashSet;

#[derive(Default)]
struct Board(HashSet<(i8, i8)>);

impl Board {
    fn new(v: &[(i8, i8)]) -> Self {
        Board(v.iter().cloned().collect())
    }

    fn modified<F>(&self, f: F) -> Self
        where F: Fn((i8, i8)) -> (i8, i8)
    {
        Board(self.0.iter().cloned().map(f).collect())
    }

    fn shifted(&self, x: i8, y: i8) -> Self {
        self.modified(|(ox, oy)| (ox + x, oy + y))
    }

    fn merged(&self, other: &Board) -> Self {
        Self(self.0.union(&other.0).cloned().collect())
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

                if let Some(_) = self.0.get(&(x as i8, y as i8)) {
                    let code = [1.0, 0.0, 0.0, 1.0];
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

#[derive(Default)]
struct Game {
    board: Board,
    metrics: Metrics,
    possible_pieces: Vec<Board>,
}

impl Game {
    fn new(metrics: Metrics) -> Self {
        Self {
            metrics,
            board: Default::default(),
            possible_pieces: vec![
                Board::new(&[
                    (0, 0),
                    (0, 1),
                    (1, 0),
                    (1, 1),
                ][..]),
                Board::new(&[
                    (0, 0),
                    (1, 0),
                    (1, 1),
                    (2, 0),
                ][..]),
            ]
        }
    }

    fn render(&self, window: &mut PistonWindow, event: &Event) {
        window.draw_2d(event, |c, g, _| {
            self.board.render(&self.metrics, &c, g);
        });
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

    game.board = game.board.merged(&game.possible_pieces[0]);
    game.board = game.board.merged(&game.possible_pieces[1].shifted(3, 3));

    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            game.render(&mut window, &e);
        }
    }
}
