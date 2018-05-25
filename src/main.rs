extern crate piston_window;
extern crate opengl_graphics;
extern crate rand;

use piston_window::*;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use opengl_graphics::GlGraphics;

#[derive(Copy, Clone)]
enum Color {
    Red, Green, Blue, Magenta, Cyan, Yellow, Orange,
}

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

#[derive(Clone)]
struct Cell {
    content: Option<Color>,
}

impl Cell {
    fn is_none(&self) -> bool {
        self.content.is_none()
    }
}

#[derive(Clone)]
struct Board {
    dim_x: usize,
    dim_y: usize,
    cells: HashMap<(isize, isize), Cell>,
}

type Piece = Board;

enum DrawEffect<'a> {
    None,
    Flash(&'a Vec<usize>),
    Darker,
}

impl Board {
    fn empty(dim_x: usize, dim_y: usize) -> Self {
        let mut cells = HashMap::new();

        for i in 0 .. dim_x as isize {
            for j in 0.. dim_y as isize {
                cells.insert((i, j), Cell { content: None });
            }
        }

        Board { dim_x, dim_y, cells, }
    }

    fn piece(spec: &[[u8; 4]; 4], color: Color) -> Self {
        let mut cells = HashMap::new();

        for x in 0.. spec[0].len() as isize {
            for y in 0 .. spec.len() as isize {
                cells.insert((x, y), Cell { content:
                    if spec[y as usize][x as usize] != 0 { Some(color) } else { None }
                });
            }
        }

        Board {
            dim_x: spec[0].len(),
            dim_y: spec.len(), cells,
        }
    }

    fn as_merged(&self, offset: (isize, isize), board: &Board) -> Option<Board> {
        let mut copy = self.clone();

        for x in 0..board.dim_x as isize {
            for y in 0..board.dim_y as isize {
                let cell = board.cells.get(&(x, y)).unwrap();
                if cell.content.is_some() {
                    let x = x + offset.0;
                    let y = y + offset.1;
                    let coords = (x, y);

                    if self.cells.get(&coords)?.content.is_none() {
                        copy.cells.insert(coords, cell.clone());
                    } else { // Collision
                        return None;
                    }
                }
            }
        }

        Some(copy)
    }

    fn draw<'a>(&self, c: &Context, gl: &mut GlGraphics, effect: DrawEffect<'a>,
                metrics: &Metrics) {
        let mut draw = |color, rect: [f64; 4]| {
            Rectangle::new(color).draw(rect, &DrawState::default(), c.transform, gl);
        };

        for x in 0..self.dim_x as isize {
            for y in 0..self.dim_y as isize {
                let block_pixels = metrics.block_pixels as f64;
                let boarder_size = block_pixels / 20.0;
                let outer = [block_pixels * (x as f64), block_pixels * (y as f64), block_pixels, block_pixels];
                let inner = [outer[0] + boarder_size, outer[1] + boarder_size,
                       outer[2] - boarder_size * 2.0, outer[3] - boarder_size * 2.0];

                draw([0.2, 0.2, 0.2, 1.0], outer);
                draw([0.1, 0.1, 0.1, 1.0], inner);

                self.cells.get(&(x, y)).unwrap().content.map(|color| {
                    let code = match color {
                        Color::Red     => [1.0, 0.0, 0.0, 1.0],
                        Color::Green   => [0.0, 1.0, 0.0, 1.0],
                        Color::Blue    => [0.5, 0.5, 1.0, 1.0],
                        Color::Magenta => [1.0, 0.0, 1.0, 1.0],
                        Color::Cyan    => [0.0, 1.0, 1.0, 1.0],
                        Color::Yellow  => [1.0, 1.0, 0.0, 1.0],
                        Color::Orange  => [1.0, 0.5, 0.0, 1.0],
                    };

                    draw(code, outer);

                    let code = [
                        code[0]*0.8,
                        code[1]*0.8,
                        code[2]*0.8,
                        code[3]
                    ];

                    draw(code, inner);
                });

                match effect {
                    DrawEffect::None => {},
                    DrawEffect::Flash(lines) => {
                        if lines.contains(&(y as usize)) {
                            draw([1.0, 1.0, 1.0, 0.5], outer);
                        }
                    }
                    DrawEffect::Darker => {
                        draw([0.0, 0.0, 0.0, 0.9], outer);
                    }
                }
            }
        }
    }

    fn does_line_satisify<F>(&self, idx: usize, f: F) -> bool
        where F: Fn(&Cell) -> bool
    {
        for x in 0..self.dim_x as isize {
            if !f(self.cells.get(&(x as isize, idx as isize)).unwrap()) {
                return false
            }
        }

        true
    }

    fn without_line(&self, idx: usize) -> Self {
        let mut cells = HashMap::new();

        for x in 0..self.dim_x as isize {
            for y in 0..self.dim_y as isize - 1 {
                let item = if y >= idx as isize {
                    self.cells.get(&(x, y + 1))
                } else {
                    self.cells.get(&(x, y))
                }.unwrap();

                cells.insert((x, y), item.clone());
            }
        }

        Board {
            dim_x : self.dim_x,
            dim_y : self.dim_y - 1,
            cells,
        }
    }

    fn prepend_empty_line(&self) -> Self {
        let mut cells = HashMap::new();

        for x in 0..self.dim_x as isize {
            cells.insert((x, 0), Cell { content: None });
        }

        for ((x, y), item) in &self.cells {
            cells.insert((*x, *y + 1), item.clone());
        }

        Board {
            dim_x : self.dim_x,
            dim_y : self.dim_y + 1,
            cells,
        }
    }

    fn with_eliminate_lines(&self, lines: &Vec<usize>) -> Self {
        let mut board = self.clone();

        for idx in lines {
            board = board.without_line(*idx);
        }

        for _ in 0..lines.len() {
            board = board.prepend_empty_line();
        }

        board
    }

    fn with_trimmed_lines(&self) -> Self {
        let mut board = self.clone();

        while board.does_line_satisify(0, Cell::is_none) {
            board = board.without_line(0);
        }

        while board.does_line_satisify(board.dim_y - 1, Cell::is_none) {
            board = board.without_line(board.dim_y - 1);
        }

        board
    }

    fn get_full_lines(&self) -> Vec<usize> {
        let mut v = vec![];

        for i in (0..self.dim_y).rev() {
            if self.does_line_satisify(i, |cell| !cell.is_none()) {
                v.push(i);
            }
        }

        v
    }

    fn transposed(&self) -> Self {
        let mut cells = HashMap::new();

        for ((x, y), item) in &self.cells {
            cells.insert((*y, *x), item.clone());
        }

        Board {
            dim_x : self.dim_y,
            dim_y : self.dim_x,
            cells,
        }
    }

    fn with_mirrored_y(&self) -> Self {
        let mut cells = HashMap::new();

        for ((x, y), item) in &self.cells {
            cells.insert((*x, self.dim_y as isize - *y - 1), item.clone());
        }

        Board {
            dim_y : self.dim_y,
            dim_x : self.dim_x,
            cells,
        }
    }

    fn with_rotated_counter(&self) -> Self {
        self.transposed().with_mirrored_y()
    }

    fn with_rotated(&self) -> Self {
        self.with_mirrored_y().transposed()
    }

    fn with_trim_sides(&self) -> Self {
        self.with_trimmed_lines().transposed().with_trimmed_lines().transposed()
    }
}

struct Falling {
    offset: (isize, isize),
    piece: Piece,
    time_since_fall: Instant,
}

enum State {
    Falling(Falling),
    Flashing(isize, Instant, Vec<usize>),
    GameOver,
}

struct Game {
    board: Board,
    metrics: Metrics,
    possible_pieces: Vec<Board>,
    state: State,
}

impl Game {
    fn new(metrics: Metrics) -> Self {
        let __ = 0;
        let xx = 01;
        let possible_pieces = vec![
            Board::piece(&[[__, __, __, __],
                           [__, xx, xx, xx],
                           [__, xx, __, __],
                           [__, __, __, __]], Color::Orange),

            Board::piece(&[[__, __, __, __],
                           [__, xx, xx, xx],
                           [__, __, __, xx],
                           [__, __, __, __]], Color::Yellow),

            Board::piece(&[[__, __, __, __],
                           [xx, xx, xx, xx],
                           [__, __, __, __],
                           [__, __, __, __]], Color::Blue),

            Board::piece(&[[xx, xx, xx, __],
                           [__, xx, __, __],
                           [__, __, __, __],
                           [__, __, __, __]], Color::Green),

            Board::piece(&[[__, __, __, __],
                           [__, xx, xx, __],
                           [xx, xx, __, __],
                           [__, __, __, __]], Color::Cyan),

            Board::piece(&[[__, __, __, __],
                           [xx, xx, __, __],
                           [__, xx, xx, __],
                           [__, __, __, __]], Color::Magenta),

            Board::piece(&[[__, __, __, __],
                           [__, xx, xx, __],
                           [__, xx, xx, __],
                           [__, __, __, __]], Color::Red),
        ].into_iter().map(|x| x.with_trim_sides()).collect();

        Game {
            board: Board::empty(metrics.board_x, metrics.board_y),
            state: State::Falling(Self::new_falling(&possible_pieces)),
            possible_pieces,
            metrics,
        }
    }

    fn new_falling(possible_pieces: &Vec<Board>) -> Falling {
        let idx = rand::random::<usize>() % possible_pieces.len();

        Falling {
            offset: (0, 0),
            piece: possible_pieces[idx].clone(),
            time_since_fall: Instant::now(),
        }
    }

    fn move_piece(&mut self, change: (isize, isize)) {
        let opt_new_state = match &mut self.state {
            State::GameOver | State::Flashing (_, _, _) => None,
            State::Falling(falling) => {
                let new_offset = {
                    let (x, y) = falling.offset;
                    ((x as isize + change.0), (y as isize + change.1))
                };
                let is_down = change == (0, 1);

                match self.board.as_merged(new_offset, &falling.piece) {
                    None => { // There were collisions
                        if is_down {
                            match self.board.as_merged(falling.offset, &falling.piece) {
                                None => Some(State::GameOver),
                                Some(merged_board) => {
                                    let completed = merged_board.get_full_lines();

                                    self.board = merged_board;
                                    *falling = Self::new_falling(&self.possible_pieces);
                                    if completed.len() > 0 {
                                        Some(State::Flashing(0, Instant::now(), completed))
                                    } else {
                                        None
                                    }
                                }
                            }
                        } else {
                            None
                        }
                    },
                    Some(_) => {
                        falling.offset = new_offset;
                        if is_down {
                            falling.time_since_fall = Instant::now();
                        }
                        None
                    }
                }
            }
        };

        if let Some(new_state) = opt_new_state {
            self.state = new_state;
        }
    }

    fn rotate(&mut self, counter: bool) {
        match &mut self.state {
            State::GameOver | State::Flashing (_, _, _) => {},
            State::Falling(falling) => {
                let rotated_piece = if counter {
                    falling.piece.with_rotated()
                } else {
                    falling.piece.with_rotated_counter()
                };

                self.board.as_merged(falling.offset, &rotated_piece).map(|_| {
                    falling.piece = rotated_piece;
                });
            }
        }
    }

    fn progress(&mut self) {
        enum Disposition {
            ShouldFall,
            NewPiece(Board),
        }

        let disp = match &mut self.state {
            State::GameOver => return,
            State::Flashing(stage, last_stage_switch, lines) => {
                if last_stage_switch.elapsed() <= Duration::from_millis(50) {
                    return;
                }
                if *stage < 18 {
                    *stage += 1;
                    *last_stage_switch = Instant::now();
                    return;
                } else {
                    Disposition::NewPiece(self.board.with_eliminate_lines(lines))
                }
            }
            State::Falling(falling) => {
                if falling.time_since_fall.elapsed() <= Duration::from_millis(700) {
                    return;
                }
                Disposition::ShouldFall
            }
        };

        match disp {
            Disposition::ShouldFall => {
                self.move_piece((0, 1));
            }
            Disposition::NewPiece(new_board) => {
                self.board = new_board;
                self.state = State::Falling(Self::new_falling(&self.possible_pieces));
            }
        }
    }

    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        let res = self.metrics.resolution();
        let ref c = Context::new_abs(res[0] as f64, res[1] as f64);

        gl.draw(args.viewport(), |_, gl| {
            match &self.state {
                State::Flashing(stage, _, lines) => {
                    let effect = {
                        if *stage % 2 == 0 {
                            DrawEffect::None
                        } else {
                            DrawEffect::Flash(&lines)
                        }
                    };
                    self.board.draw(c, gl, effect, &self.metrics);
                }
                State::Falling(falling) => {
                    match self.board.as_merged(falling.offset, &falling.piece) {
                        None => { }
                        Some(x) => {
                            x.draw(c, gl, DrawEffect::None, &self.metrics);
                        }
                    }
                }
                State::GameOver => {
                    self.board.draw(c, gl, DrawEffect::Darker, &self.metrics);
                }
            }
        });
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
            self.move_piece(movement);
            return;
        }

        match key {
            Key::Up => self.rotate(false),
            Key::NumPad5 => self.rotate(true),
            _ => return,
        }
    }
}

fn main() {
    let metrics = Metrics {
        block_pixels: 50,
        board_x: 8,
        board_y: 20,
    };

    let mut window: PistonWindow
        = WindowSettings::new("Tetris", metrics.resolution()).exit_on_esc(true).build().unwrap_or_else(
            |e| { panic!("Failed: {}", e) }
        );

    let mut gl = GlGraphics::new(OpenGL::V3_2);
    let mut game = Game::new(metrics);

    while let Some(e) = window.next() {
        game.progress();

        if let Some(args) = e.render_args() {
            game.render(&mut gl, &args);
        }

        if let Some(args) = e.press_args() {
            game.on_press(&args);
        }
    }
}
