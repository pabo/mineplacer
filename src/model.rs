use pagurus::{
    random::StdRng,
    spatial::{Contains, Position, Region, Size},
    Result, System,
};
// use rand::seq::SliceRandom;
use std::time::Duration;

const WIDTH: usize = 30;
const HEIGHT: usize = 30;

#[derive(Debug, Default, Clone, Copy)]
pub enum Level {
    Small,
    #[default]
    Large,
}

impl Level {
    // fn mines(self) -> usize {
    //     match self {
    //         Level::Small => 50,
    //         Level::Large => 99,
    //     }
    // }

    fn width(self) -> usize {
        match self {
            Level::Small => 8,
            Level::Large => 30,
        }
    }

    fn height(self) -> usize {
        match self {
            Level::Small => 15,
            Level::Large => 30,
        }
    }

    fn offset(self) -> Position {
        match self {
            Level::Small => Position::from_xy(4, 7),
            Level::Large => Position::from_xy(0, 0),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum State {
    #[default]
    Initial,
    Playing,
    Won {
        elapsed_time: Duration,
    },
}

#[derive(Debug, Default, Clone)]
pub struct Model {
    rng: StdRng,
    board: Board,
    remaining_mines: usize,
    start_time: Duration,
    elapsed_time: Duration,
    level: Level,
    state: State,
}

impl Model {
    pub fn initialize<S: System>(&mut self, system: &mut S) -> Result<()> {
        self.rng = StdRng::from_clock_seed(system.clock_unix_time());
        self.board.region.size = Size::from_wh(WIDTH as u32, HEIGHT as u32);
        Ok(())
    }

    pub fn start_game<S: System>(&mut self, system: &mut S, level: Level) -> Result<()> {
        self.level = level;
        self.board = Board::default();

        self.board.region = Region::new(
            level.offset(),
            Size::from_wh(level.width() as u32, level.height() as u32),
        );

        // f.u. wf
        let rows = [
            "                              ",
            "                              ",
            "  XXXX  X   X   XXXX  X  X    ",
            "  X     X   X  X      X X     ",
            "  XXX   X   X  X      XX      ",
            "  X     X   X  X      X X     ",
            "  X      XXX    XXXX  X  X    ",
            "                              ",
            "  X     X   XXX   X   X       ",
            "  X     X  X   X  XX  X       ",
            "  X  X  X  XXXXX  X X X       ",
            "  X X X X  X   X  X  XX       ",
            "   X   X   X   X  X   X       ",
            "                              ",
            "  XXXX  X   X  X   X   XXX    ",
            "  X     X   X  XX  X  X       ",
            "  XXX   X   X  X X X  X XXX   ",
            "  X     X   X  X  XX  X   X   ",
            "  X      XXX   X   X   XXX    ",
        ];

        self.remaining_mines = 0;
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.split("").filter(|x| !x.is_empty()).enumerate() {
                if c == "X" {
                    self.board.cells[y as usize][x as usize].expected_mine = true;
                    self.remaining_mines = self.remaining_mines + 1;
                }
            }
        }

        // // create a vector of all the positions (in order)
        // let mut mines = self.board.region.iter().collect::<Vec<_>>();

        // // randomize the vector
        // mines.shuffle(&mut self.rng);

        // // for the first (number of mines desired) items in vector, mark the board cells as mines
        // for p in &mines[0..level.mines()] {
        //     self.board.cells[p.y as usize][p.x as usize].expected_mine = true;
        // }

        // self.remaining_mines = level.mines();

        self.start_time = system.clock_game_time();
        self.state = State::Playing;
        Ok(())
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn remaining_mines(&self) -> usize {
        self.remaining_mines
    }

    pub fn update_elapsed_time<S: System>(&mut self, system: &S) {
        self.elapsed_time = system.clock_game_time() - self.start_time;
    }

    pub fn elapsed_time(&self) -> Duration {
        self.elapsed_time
    }

    pub fn surrounding_mines(&self) -> impl '_ + Iterator<Item = (Position, isize)> {
        self.board
            .region
            .iter()
            .map(|p| (p, self.board.surrounding_mines(p)))
    }

    pub fn handle_click(&mut self, position: Position) {
        if self.state != State::Playing {
            return;
        }
        if !self.board.region.contains(&position) {
            return;
        }

        let cell = &mut self.board.cells[position.y as usize][position.x as usize];

        if !cell.actual_mine && self.remaining_mines == 0 {
            return;
        }

        cell.actual_mine = !cell.actual_mine;
        if cell.actual_mine {
            self.remaining_mines -= 1;
        } else {
            self.remaining_mines += 1;
        }

        if self.remaining_mines == 0 && self.surrounding_mines().all(|(_, m)| m == 0) {
            self.state = State::Won {
                elapsed_time: self.elapsed_time(),
            }
        }
    }

    pub fn has_mine(&self, p: Position) -> bool {
        self.board.cells[p.y as usize][p.x as usize].actual_mine
    }
}

#[derive(Debug, Default, Clone)]
struct Board {
    cells: [[Cell; WIDTH]; HEIGHT],
    region: Region,
}

impl Board {
    fn surrounding_mines(&self, p: Position) -> isize {
        let mut expected: isize = 0;
        let mut actual: isize = 0;
        for y_delta in [-1, 0, 1] {
            for x_delta in [-1, 0, 1] {
                let p = p.move_y(y_delta).move_x(x_delta);
                if !self.region.contains(&p) {
                    continue;
                }

                let cell = self.cells[p.y as usize][p.x as usize];
                if cell.expected_mine {
                    expected += 1;
                }
                if cell.actual_mine {
                    actual += 1;
                }
            }
        }
        expected - actual
    }
}

#[derive(Debug, Default, Clone, Copy)]
struct Cell {
    expected_mine: bool,
    actual_mine: bool,
}
