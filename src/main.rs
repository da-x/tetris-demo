use piston_window::*;
use std::time::{Instant, Duration};

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

type Cell = Option<Color>;

#[derive(Clone)]
struct Board {
    cells: Vec<Vec<Cell>>,
}

type Piece = Board;

enum DrawEffect<'a> {
    None,
    Flash(&'a Vec<usize>),
    Darker,
}

impl Board {
    fn empty(dim_x: usize, dim_y: usize) -> Self {
        let line : Vec<_> = (0..dim_x).map(|_|None).collect();
        let cells : Vec<_> = (0..dim_y).map(|_|line.clone()).collect();
        Board { cells }
    }

    fn valid(&self, offset: (isize, isize)) -> bool {
        if offset.0 >= 0  &&  offset.0 < self.dim_x() as isize {
            if offset.1 >= 0  &&  offset.1 < self.dim_y() as isize {
                return true;
            }
        }

        return false;
    }

    fn dim_x(&self) -> usize { self.cells[0].len() }
    fn dim_y(&self) -> usize { self.cells.len() }

    fn piece(spec: &[[u8; 4]; 4], color: Color) -> Self {
        let mut board = Board::empty(spec[0].len(), spec.len());

        for x in 0.. spec[0].len() {
            for y in 0 .. spec.len() {
                board.cells[y][x] = if spec[y][x] != 0 { Some(color) } else { None }
            }
        }

        board
    }

    fn as_merged(&self, offset: (isize, isize), board: &Board) -> Option<Board> {
        let mut copy = self.clone();

        for x in 0..board.dim_x() {
            for y in 0..board.dim_y() {
                let cell = board.cells[y][x];
                if cell.is_some() {
                    let x = x as isize + offset.0;
                    let y = y as isize + offset.1;
                    if !self.valid((x, y)) {
                        return None;
                    }
                    if self.cells[y as usize][x as usize].is_none() {
                        copy.cells[y as usize][x as usize] = cell.clone();
                    } else { // Collision
                        return None;
                    }
                }
            }
        }

        Some(copy)
    }

    fn draw<'a, G>(&self, c: &Context, g: &mut G, effect: DrawEffect<'a>,
                metrics: &Metrics)
        where G: Graphics
    {
        let mut draw = |color, rect: [f64; 4]| {
            Rectangle::new(color).draw(rect, &DrawState::default(), c.transform, g);
        };

        for x in 0..self.dim_x() {
            for y in 0..self.dim_y() {
                let block_pixels = metrics.block_pixels as f64;
                let border_size = block_pixels / 20.0;
                let outer = [block_pixels * (x as f64), block_pixels * (y as f64), block_pixels, block_pixels];
                let inner = [outer[0] + border_size, outer[1] + border_size,
                       outer[2] - border_size * 2.0, outer[3] - border_size * 2.0];

                draw([0.2, 0.2, 0.2, 1.0], outer);
                draw([0.1, 0.1, 0.1, 1.0], inner);

                self.cells[y][x].map(|color| {
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

    fn without_line(&self, idx: usize) -> Self {
        let mut board = self.clone();

        board.cells.remove(idx);
        board
    }

    fn prepend_empty_line(&self) -> Self {
        let line : Vec<_> = (0..self.dim_x()).map(|_|None).collect();
        let mut board = self.clone();

        board.cells.insert(0, line);
        board
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

        while board.cells[0].iter().all(Cell::is_none) {
            board = board.without_line(0);
        }

        while board.cells[board.dim_y() - 1].iter().all(Cell::is_none) {
            board = board.without_line(board.dim_y() - 1);
        }

        board
    }

    fn get_full_lines_indicts(&self) -> Vec<usize> {
        self.cells.iter().enumerate()
            .rev().filter(|(_, line)| line.iter().all(|cell| !cell.is_none()))
            .map(|(idx, _)| idx).collect()
    }

    fn transposed(&self) -> Self {
        let mut board = Self::empty(self.dim_y(), self.dim_x());

        for x in 0..self.dim_x() {
            for y in 0..self.dim_y() {
                board.cells[x][y] = self.cells[y][x];
            }
        }

        board
    }

    fn with_mirrored_y(&self) -> Self {
        let mut board = Self::empty(self.dim_x(), self.dim_y());

        for x in 0..self.dim_x() {
            for y in 0..self.dim_y() {
                board.cells[y][x] = self.cells[y][self.dim_x() - x - 1];
            }
        }

        board
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

                if self.board.as_merged(new_offset, &falling.piece).is_none() {
                     // There were collisions
                    if is_down {
                        match self.board.as_merged(falling.offset, &falling.piece) {
                            None => Some(State::GameOver),
                            Some(merged_board) => {
                                let completed = merged_board.get_full_lines_indicts();
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
                } else { // Keep falling
                    falling.offset = new_offset;
                    if is_down {
                        falling.time_since_fall = Instant::now();
                    }
                    None
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
            Disposition::ShouldFall => self.move_piece((0, 1)),
            Disposition::NewPiece(new_board) => {
                self.board = new_board;
                self.state = State::Falling(Self::new_falling(&self.possible_pieces));
            }
        }
    }

    fn render(&self, window: &mut PistonWindow, event: &Event)
    {
        window.draw_2d(event, |c, g, _| {
            match &self.state {
                State::Flashing(stage, _, lines) => {
                    let effect = {
                        if *stage % 2 == 0 {
                            DrawEffect::None
                        } else {
                            DrawEffect::Flash(&lines)
                        }
                    };
                    self.board.draw(&c, g, effect, &self.metrics);
                }
                State::Falling(falling) => {
                    if let Some(merged) = self.board.as_merged(falling.offset, &falling.piece) {
                        merged.draw(&c, g, DrawEffect::None, &self.metrics);
                    }
                }
                State::GameOver => {
                    self.board.draw(&c, g, DrawEffect::Darker, &self.metrics);
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
        block_pixels: 20,
        board_x: 8,
        board_y: 20,
    };

    let mut window: PistonWindow = WindowSettings::new("Tetris",
            metrics.resolution()).exit_on_esc(true).build().unwrap_or_else(
                |e| { panic!("Failed: {}", e) }
            );

    let mut game = Game::new(metrics);

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
