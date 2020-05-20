use piston_window::{WindowSettings, PistonWindow, Event, RenderEvent};
use piston_window::{Rectangle, DrawState, Context, Graphics};

use std::collections::HashSet;

#[derive(Default)]
struct Board(HashSet<(i8, i8)>);

impl Board {
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
}

impl Game {
    fn new(metrics: Metrics) -> Self {
        Self {
            metrics,
            board: Default::default(),
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
    let game = Game::new(metrics);

    while let Some(e) = window.next() {
        if let Some(_) = e.render_args() {
            game.render(&mut window, &e);
        }
    }
}
