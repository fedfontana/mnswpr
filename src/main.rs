#![allow(dead_code, unused)]

use std::fmt::Display;
use std::io::{Write, stdout, stdin};

use rand::{seq::SliceRandom, thread_rng, Rng};

use termion::event::{Key, Event, MouseEvent};
use termion::input::{TermRead, MouseTerminal};
use termion::raw::IntoRawMode;
use termion::color;

#[derive(Copy, Clone, Debug)]
pub enum CellState {
    Open,
    Closed,
    Flagged,
}

impl Default for CellState {
    fn default() -> Self {
        Self::Closed
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CellContent {
    Mine,
    Empty,
}

struct Cursor {
    row: usize,
    col: usize,
}

impl Default for CellContent {
    fn default() -> Self {
        Self::Empty
    }
}

const NBOR_COUNT_TO_FG_COLOR: [color::Fg<&'static dyn color::Color>; 9] = [
                                            color::Fg(&color::Reset),       // 0
                                            color::Fg(&color::LightBlue),   // 1
                                            color::Fg(&color::LightBlue),   // 2
                                            color::Fg(&color::Blue),        // 3
                                            color::Fg(&color::Blue),        // 4
                                            color::Fg(&color::LightRed),    // 5
                                            color::Fg(&color::LightRed),    // 6
                                            color::Fg(&color::Red),         // 7
                                            color::Fg(&color::Red),         // 8
                                            ];

const BG_COLOR: color::Bg<color::Rgb> = color::Bg(color::Rgb(40, 40, 40));

#[derive(Copy, Clone, Debug, Default)]
pub struct Cell {
    state: CellState,
    content: CellContent,
    neighboring_bomb_count: usize,
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.state {
            CellState::Open => match self.content {
                CellContent::Mine => write!(f, "{}*{}", color::Bg(color::Red), color::Bg(color::Reset)),
                CellContent::Empty => write!(f, "{}{}{}{}{}", 
                    BG_COLOR, 
                    NBOR_COUNT_TO_FG_COLOR[self.neighboring_bomb_count], 
                    self.neighboring_bomb_count, 
                    color::Fg(color::Reset), 
                    color::Bg(color::Reset)
                ),
            },
            CellState::Closed => write!(f, "."),
            CellState::Flagged => write!(f, "{}F{}", color::Bg(color::Blue), color::Bg(color::Reset)),
        }
    }
}

impl Cell {
    fn set_state(&mut self, new_state: CellState) {
        self.state = new_state;
    }
}

struct Field {
    rows: usize,
    cols: usize,
    grid: Vec<Cell>,
    cursor: Cursor,
    uncovered_empty_cells: usize,
}

impl Field {
    fn new(rows: usize, cols: usize, mut bomb_percentage: usize) -> Self {
        if bomb_percentage > 100 {
            bomb_percentage = 100;
        }

        let mut grid = Vec::with_capacity(rows * cols);
        let mut uncovered_empty_cells = 0;

        // Generate random board
        let mut rng = thread_rng();
        for idx in 0..rows * cols {
            let cell_content = if rng.gen_range(1..=100) <= bomb_percentage {
                CellContent::Mine
            } else {
                uncovered_empty_cells += 1;
                CellContent::Empty
            };

            let cell = Cell {
                state: CellState::Closed,
                content: cell_content,
                neighboring_bomb_count: 0,
            };

            grid.push(cell);
        }

        // Update the counters of the neighboring mines for each mine
        for idx in 0..rows * cols {
            let mut count = 0;
            let row = (idx / cols) as isize;
            let col = (idx % cols) as isize;
            for delta_row in -1..=1 {
                for delta_col in -1..=1 {
                    // Do not count the current cell
                    if delta_col == 0 && delta_row == 0 {
                        continue;
                    }

                    let current_row  = row+delta_row;
                    let current_col = col+delta_col;
                    // Do not consider out of bounds cells
                    if current_row >= rows as isize
                        || current_col >= cols as isize
                        || current_row < 0
                        || current_col < 0
                    {
                        continue;
                    }

                    match grid[current_row as usize * cols + current_col as usize]
                        .content
                    {
                        CellContent::Mine => count += 1,
                        CellContent::Empty => {}
                    }
                }
            }
            grid[row as usize*cols+col as usize].neighboring_bomb_count = count;
        }

        Self { rows, cols, grid, cursor: Cursor { row: 0, col: 0 }, uncovered_empty_cells}
    }

    fn get(&self, row: usize, col: usize) -> Option<Cell> {
        if row > self.rows || col > self.cols {
            return None;
        }
        Some(self.grid[row * self.cols + col])
    }

    fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        if row > self.rows || col > self.cols {
            return None;
        }
        Some(&mut self.grid[row * self.cols + col])
    }

    fn set(&mut self, row: usize, col: usize, new_value: Cell) -> Option<Cell> {
        let old_val = self.get(row, col);
        if old_val.is_none() {
            return None;
        }
        self.grid[row * self.cols + col] = new_value;
        Some(old_val.unwrap())
    }

    fn uncover_at_cursor(&mut self) -> CellContent {
        let cell_under_cursor = self.get_mut(self.cursor.row, self.cursor.col).unwrap();
        let old_content = cell_under_cursor.content.clone();

        // if state is Open or Flagged, do nothing
        if let CellState::Closed = cell_under_cursor.state {
            cell_under_cursor.set_state(CellState::Open);
            match cell_under_cursor.content {
                CellContent::Mine => {
                    // Do nothing, the caller will do stuff with the return value
                },
                CellContent::Empty => {
                    //TODO open neighbors if they have 0 bombs?
                    self.uncovered_empty_cells -= 1;
                },
            }
        }

        old_content
    }

    fn toggle_flag_at_cursor(&mut self) {
        let cell_under_cursor = self.get_mut(self.cursor.row, self.cursor.col).unwrap();
        match cell_under_cursor.state {
            CellState::Open => {},
            CellState::Closed =>  cell_under_cursor.set_state(CellState::Flagged),
            CellState::Flagged => cell_under_cursor.set_state(CellState::Closed),
        };
    }

    fn uncover_all(&mut self) {
        self.grid.iter_mut().for_each(|cell| cell.state = CellState::Open);
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_repr = String::new(); //TODO use with_capacity()?

        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = self.get(row, col).unwrap();
                if self.cursor.row == row && self.cursor.col == col {
                    str_repr = format!("{BG_COLOR}{str_repr}[{cell}]{}", color::Bg(color::Reset));
                } else {
                    str_repr = format!("{BG_COLOR}{str_repr} {cell} {}", color::Bg(color::Reset));
                }
            }
            str_repr = format!("{str_repr}\r\n");
        }

        write!(f, "{}", str_repr)
    }
}

//TODO fix the board continously going down with time
//TODO add functionality to uncover the whole set of 0s when clicking on one of them
//TODO add mouse click support (supported by termion)

//TODO add cli options:
//  - widht
//  - height
//  - bomb_percentage
//  - presets like easy, hard, medium, ...
//  - auto-counter or something like that. When this option is active, numbers that have too many flags around them turn bright red,
//          and the ones with the right amount of flags around them get printed green

//TODO add a retry option
//TODO generate board when clicking on first cell. Either generate a number or a whole area of numbers under the cursor in a way that the first tile cannot be a bomb.

//TODO fix the colors not covering the whole board
//TODO add more different colors for each bomb count
//TODO when a match is lost, highlight the flags in the wrong place with a different color
//TODO print the number of the remaining bombs
//TODO do not allow flagging when current_flag_count > bomb_count
fn main() {
    let mut field = Field::new(10, 10, 20);

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(stdout, "{}{}{field}",termion::clear::All, termion::cursor::Goto(1,1 ));
    stdout.flush().unwrap();

    for c in stdin.events() {
        if let Event::Key(event) = c.unwrap() {
            match event {
                Key::Char('q') | Key::Char('Q') => break,
                Key::Char('w') | Key::Char('W') | Key::Up => {
                    if field.cursor.row > 0 {
                        field.cursor.row -= 1;
                    }
                },
                Key::Char('a') | Key::Char('A') |  Key::Left => {
                    if field.cursor.col > 0 {
                        field.cursor.col -= 1;
                    }
                },
                Key::Char('s') | Key::Char('S') | Key::Down => {
                    if field.cursor.row < field.rows-1 {
                        field.cursor.row += 1;
                    }
                },
                Key::Char('d') | Key::Char('D') | Key::Right => {
                    if field.cursor.col < field.cols-1{
                        field.cursor.col += 1;
                    }
                },
                Key::Char(' ') => {
                    match field.uncover_at_cursor() {
                        CellContent::Mine => {
                            let cell = field.get(field.cursor.row, field.cursor.col).unwrap();
                            if !matches!(cell.state, CellState::Flagged) { 
                                field.uncover_all();
                                write!(stdout, "{}{}{field}",termion::clear::All, termion::cursor::Goto(1,1));
                                println!("{}You lost!{}", color::Fg(color::LightRed), color::Fg(color::Reset));
                                break;
                            }
                        },
                        CellContent::Empty => {},
                    }
                },
                Key::Char('f') | Key::Char('F') => field.toggle_flag_at_cursor(),
                _ => {},
            }
        }
        write!(stdout, "{}{field}",termion::clear::All);
        if field.uncovered_empty_cells == 0 {
            write!(stdout, "{}You won!{}",color::Fg(color::Green), color::Fg(color::Reset));
            stdout.flush().unwrap();
            break;
        }
        stdout.flush().unwrap();

    }
}