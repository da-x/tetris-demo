extern crate piston_window;
extern crate opengl_graphics;
extern crate sdl2_window;

use piston_window::*;
use sdl2_window::Sdl2Window;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use opengl_graphics::GlGraphics;

#[derive(Debug, Copy, Clone)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Clone)]
struct Cell {
    content: Option<Color>,
}

#[derive(Clone)]
struct Board {
    dim_x: usize,
    dim_y: usize,
    cells: HashMap<(usize, usize), Cell>,
}

type Piece = Board;

impl Board {
    fn empty(dim_x: usize, dim_y: usize) -> Self {
        let mut cells = HashMap::new();

        for i in 0 .. dim_x {
            for j in 0.. dim_y {
                cells.insert((i, j), Cell { content: None });
            }
        }

        Board {
            dim_x, dim_y, cells,
        }
    }

    fn piece(spec: &[[u8; 4]; 4], color: Color) -> Self {
        let mut cells = HashMap::new();

        for x in 0.. spec[0].len() {
            for y in 0 .. spec.len() {
                cells.insert((x, y), Cell { content:
                    if spec[y][x] != 0 { Some(color) } else { None }
                });
            }
        }

        Board {
            dim_x: spec[0].len(),
            dim_y: spec.len(), cells,
        }
    }

    fn check_mergeable(&self, offset:(usize, usize), board: &Board) -> Option<Board> {
        let mut copy = self.clone();

        for x in 0..board.dim_x {
            for y in 0..board.dim_y {
                if let Some(cell) = board.cells.get(&(x, y)) {
                    if cell.content.is_some() {
                        let x = x + offset.0;
                        let y = y + offset.1;

                        if let Some(my_cell) = self.cells.get(&(x, y)) {
                            if my_cell.content.is_none() {
                                copy.cells.insert((x, y), cell.clone());
                            } else {
                                // Collision
                                return None;
                            }
                        } else {
                            // Overflow from screen
                            return None;
                        }
                    }
                }
            }
        }

        Some(copy)
    }

    fn draw(&self, c: &Context, gl: &mut GlGraphics) {
        let mut draw = |color, rect: [f64; 4]| {
            Rectangle::new(color).draw(rect,
                    &DrawState::default(), c.transform, gl);
        };
        for x in 0..self.dim_x {
            for y in 0..self.dim_y {
                let outer
                    = [20.0 * (x as f64), 20.0 * (y as f64), 20.0, 20.0];
                let inner
                    = [outer[0] + 1.0, outer[1] + 1.0,
                       outer[2] - 2.0, outer[3] - 2.0];

                draw([0.2, 0.2, 0.2, 1.0], outer);
                draw([0.1, 0.1, 0.1, 1.0], inner);

                let color = {
                    self.cells.get(&(x, y)).unwrap().content.map(|_|
                        [0.4, 0.8, 0.5, 1.0])
                };

                color.map(|color| draw(color, outer));
                color.map(|color| {
                    let color = [color[0]*0.9,
                    color[1]*0.9,
                    color[2]*0.9,
                    color[3]];
                    draw(color, inner);
                });
            }
        }
    }
}

struct Falling {
    offset: (usize, usize),
    piece: Piece,
    time_since_fall: Instant,
}

enum State {
    Falling(Falling),
    GameOver,
}

struct Game {
    board: Board,
    possible_pieces: Vec<Board>,
    state: State,
}

impl Game {
    fn new(dim_x: usize, dim_y: usize) -> Self {
        let possible_pieces = vec![
            Board::piece(
                &[[1, 0, 0, 0],
                  [1, 1, 1, 0],
                  [0, 0, 0, 0],
                  [0, 0, 0, 0]], Color::Green),

            Board::piece(
                &[[0, 1, 0, 0],
                  [1, 1, 1, 0],
                  [0, 0, 0, 0],
                  [0, 0, 0, 0]], Color::Green),
        ];

        let falling = Falling {
            offset: (0, 0),
            piece: possible_pieces[0].clone(),
            time_since_fall: Instant::now(),
        };

        Game {
            board: Board::empty(dim_x, dim_y),
            state: State::Falling(falling),
            possible_pieces,
        }
    }

    fn new_falling(possible_pieces: &Vec<Board>) -> Falling {
        Falling {
            offset: (0, 0),
            piece: possible_pieces[0].clone(),
            time_since_fall: Instant::now(),
        }
    }

    fn progress(&mut self) {
        let opt_new_state = match &mut self.state {
            State::GameOver => {
                None
            }
            State::Falling(falling) => {
                if falling.time_since_fall.elapsed() > Duration::from_millis(70) {
                    let new_offset = {
                        let (x, y) = falling.offset;
                        (x, y + 1)
                    };

                    match self.board.check_mergeable(new_offset, &falling.piece) {
                        None => {
                            match self.board.check_mergeable(falling.offset, &falling.piece) {
                                None => {
                                    Some(State::GameOver)
                                }
                                Some(x) => {
                                    self.board = x;
                                    *falling = Self::new_falling(&self.possible_pieces);
                                    None
                                }
                            }
                        },
                        Some(_) => {
                            falling.offset = new_offset;
                            falling.time_since_fall = Instant::now();
                            None
                        }
                    }
                } else {
                    None
                }
            }
        };

        if let Some(new_state) = opt_new_state {
            self.state = new_state;
        }
    }
}

fn main() {
    let mut window: PistonWindow<Sdl2Window>
        = WindowSettings::new("Tetris?", [1000, 1000]).exit_on_esc(true).build().unwrap_or_else(
            |e| { panic!("Failed: {}", e) });

    let mut gl = GlGraphics::new(OpenGL::V3_2);
    let mut game = Game::new(10, 30);

    while let Some(e) = window.next() {
        game.progress();

        if let Some(ref args) = e.render_args() {
            let ref c = Context::new_abs(1000.0, 1000.0);
            gl.draw(args.viewport(), |_, gl| {
                match &game.state {
                    State::Falling(falling) => {
                        match game.board.check_mergeable(falling.offset, &falling.piece) {
                            None => { }
                            Some(x) => {
                                x.draw(c, gl);
                            }
                        }
                    }
                    State::GameOver => {
                        game.board.draw(c, gl);
                    }
                }
            });
        }

        if let Some(ref args) = e.update_args() {
        }

        if let Some(ref args) = e.press_args() {
            println!("Press: {:?}", args);
        }
    }
}
