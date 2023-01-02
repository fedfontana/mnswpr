#![allow(dead_code, unused)]

use std::fmt::Display;
use std::io::{stdin, stdout, Write};

use rand::{seq::SliceRandom, thread_rng, Rng};

use termion::color;
use termion::cursor::HideCursor;
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;

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
    color::Fg(&color::Reset),               // 0
    color::Fg(&color::Rgb(70, 70, 255)),      // 1 // original is Rgb(0, 0, 255)
    color::Fg(&color::Rgb(0, 130, 0)),      // 2
    color::Fg(&color::Rgb(200, 0, 0)),      // 3
    color::Fg(&color::Rgb(50, 50, 131)),      // 4 // original is Rgb(0, 0, 131)
    color::Fg(&color::Rgb(132, 0 ,1)),      // 5
    color::Fg(&color::Rgb(0, 130, 132)),    // 6
    color::Fg(&color::Rgb(132, 0, 132)),    // 7
    color::Fg(&color::Rgb(117, 117, 117)),  // 8
];

const BG_COLOR: color::Bg<color::Rgb> = color::Bg(color::Rgb(30, 30, 30));

#[derive(Copy, Clone, Debug, Default)]
pub struct Cell {
    state: CellState,
    content: CellContent,
    neighbouring_bomb_count: usize,
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.state {
            CellState::Open => match self.content {
                CellContent::Mine => {
                    write!(f, "{}*{}", color::Bg(color::Red), color::Bg(color::Reset))
                }
                CellContent::Empty => write!(
                    f,
                    "{}{}{}{}{}",
                    BG_COLOR,
                    NBOR_COUNT_TO_FG_COLOR[self.neighbouring_bomb_count],
                    if self.neighbouring_bomb_count != 0 { self.neighbouring_bomb_count.to_string() } else { " ".to_string() },
                    color::Fg(color::Reset),
                    color::Bg(color::Reset)
                ),
            },
            CellState::Closed => write!(f, "."),
            CellState::Flagged => {
                write!(f, "{}F{}", color::Bg(color::Blue), color::Bg(color::Reset))
            }
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
    covered_empty_cells: usize,
    mine_count: usize,
    flag_count: usize,
}

impl Field {
    fn new(rows: usize, cols: usize, mut bomb_percentage: usize) -> Self {
        if bomb_percentage > 100 {
            bomb_percentage = 100;
        }

        let mut grid = Vec::with_capacity(rows * cols);
        let mut covered_empty_cells = 0;
        let mut mine_count = 0;

        // Generate random board
        let mut rng = thread_rng();
        for idx in 0..rows * cols {
            let cell_content = if rng.gen_range(1..=100) <= bomb_percentage {
                mine_count += 1;
                CellContent::Mine
            } else {
                covered_empty_cells += 1;
                CellContent::Empty
            };

            let cell = Cell {
                state: CellState::Closed,
                content: cell_content,
                neighbouring_bomb_count: 0,
            };

            grid.push(cell);
        }

        // Update the counters of the neighbouring mines for each mine
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

                    let current_row = row + delta_row;
                    let current_col = col + delta_col;
                    // Do not consider out of bounds cells
                    if current_row >= rows as isize
                        || current_col >= cols as isize
                        || current_row < 0
                        || current_col < 0
                    {
                        continue;
                    }

                    match grid[current_row as usize * cols + current_col as usize].content {
                        CellContent::Mine => count += 1,
                        CellContent::Empty => {}
                    }
                }
            }
            grid[row as usize * cols + col as usize].neighbouring_bomb_count = count;
        }

        Self {
            rows,
            cols,
            grid,
            cursor: Cursor { row: 0, col: 0 },
            covered_empty_cells,
            mine_count,
            flag_count: 0,
        }
    }

    fn get(&self, row: usize, col: usize) -> Option<Cell> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        Some(self.grid[row * self.cols + col])
    }

    fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut Cell> {
        if row >= self.rows || col >= self.cols {
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

    fn cell_under_cursor_mut(&mut self) -> &mut Cell {
        self.get_mut(self.cursor.row, self.cursor.col).unwrap()
    }

    fn uncover_at_cursor(&mut self) -> CellContent {
        let old_content = self.cell_under_cursor_mut().content.clone();

        fn _uncover_at_cursor_rec(field: &mut Field, current_row: isize, current_col: isize) {
            if current_row < 0 || current_col < 0 {
                return;
            }
            let res = field.get(current_row as usize, current_col as usize);
            if res.is_none() {
                return;
            }
            let current_cell = res.unwrap();

            // if state is Open or Flagged, do nothing
            if !matches!(current_cell.state, CellState::Closed) {
                return;
            }

            if let CellContent::Mine = current_cell.content {
                return;
            }

            field.covered_empty_cells -= 1;
            let current_cell = field.get_mut(current_row as usize, current_col as usize).unwrap();
            current_cell.set_state(CellState::Open);

            // Do not call recursively if we are at the edge of the 0s region
            if current_cell.neighbouring_bomb_count == 0 {
                // for each neighbouring cell run the function recursively
                for drow in -1..=1 {
                    for dcol in -1..=1 {
                        if drow == 0 && dcol == 0 {
                            continue;
                        }
                        // Bounds will be checked by the next recursive call's `if let Some(..)` and the `if` before that
                        _uncover_at_cursor_rec(field, current_row + drow, current_col + dcol)
                    }
                }
            }
        }

        _uncover_at_cursor_rec(self, self.cursor.row as isize, self.cursor.col as isize);

        old_content
    }

    fn toggle_flag_at_cursor(&mut self) {
        let can_flag = self.flag_count < self.mine_count;
        let cell_under_cursor = self.get_mut(self.cursor.row, self.cursor.col).unwrap();
        match cell_under_cursor.state {
            CellState::Open => {}
            CellState::Closed => {
                if can_flag {
                    cell_under_cursor.set_state(CellState::Flagged);
                    self.flag_count += 1;
                }
            }
            CellState::Flagged => {
                cell_under_cursor.set_state(CellState::Closed);
                self.flag_count -= 1;
            }
        };
    }

    fn uncover_all(&mut self) {
        self.grid
            .iter_mut()
            .for_each(|cell| cell.state = CellState::Open);
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_repr = String::new(); //TODO use with_capacity()? / implement this in a more efficient way

        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = self.get(row, col).unwrap();
                if self.cursor.row == row && self.cursor.col == col {
                    str_repr = format!("{str_repr}{BG_COLOR}[{cell}{BG_COLOR}]{}", color::Bg(color::Reset));
                } else {
                    str_repr = format!("{str_repr}{BG_COLOR} {cell}{BG_COLOR} {}", color::Bg(color::Reset));
                }
            }
            str_repr = format!("{str_repr}\r\n");
        }

        write!(f, "{}", str_repr)
    }
}

//TODO split into files
//TODO Game struct should be different from Field struct
//TODO refactor

//TODO add mouse click support (supported by termion)?
//TODO add docs and tests
//TODO add cli options:
//  - widht
//  - height
//  - bomb_percentage
//  - presets like easy, hard, medium, ...
//  - auto-counter or something like that. When this option is active, numbers that have too many flags around them turn bright red,
//          and the ones with the right amount of flags around them get printed green

//TODO add a retry option after concluding a match
//TODO when a match is lost, highlight the flags in the wrong place with a different color
//TODO add bg color for uncovered cells (the original gray (185, 185, 185))
//TODO generate board when clicking on first cell. 
//  - Either generate a number or a whole area of numbers under the cursor in a way that the first tile cannot be a bomb.



fn main() {
    let mut field = Field::new(10, 10, 20);

    let stdin = stdin();
    let mut stdout = HideCursor::from(stdout().into_raw_mode().unwrap());

    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1,1));

    write!(
        stdout,
        "{}Mines:{}    Flags:{}\r\n{field}",
        termion::cursor::Goto(1, 1),
        field.mine_count,
        field.flag_count
    );
    stdout.flush().unwrap();

    for c in stdin.events() {
        if let Event::Key(event) = c.unwrap() {
            match event {
                Key::Char('q') | Key::Char('Q') => break,
                Key::Char('w') | Key::Char('W') | Key::Up => {
                    if field.cursor.row > 0 {
                        field.cursor.row -= 1;
                    }
                }
                Key::Char('a') | Key::Char('A') | Key::Left => {
                    if field.cursor.col > 0 {
                        field.cursor.col -= 1;
                    }
                }
                Key::Char('s') | Key::Char('S') | Key::Down => {
                    if field.cursor.row < field.rows - 1 {
                        field.cursor.row += 1;
                    }
                }
                Key::Char('d') | Key::Char('D') | Key::Right => {
                    if field.cursor.col < field.cols - 1 {
                        field.cursor.col += 1;
                    }
                }
                Key::Char(' ') => match field.uncover_at_cursor() {
                    CellContent::Mine => {
                        let cell = field.get(field.cursor.row, field.cursor.col).unwrap();
                        if !matches!(cell.state, CellState::Flagged) {
                            field.uncover_all();
                            write!(
                                stdout,
                                "{}{}Mines:{}    Flags:{}\r\n{field}",
                                termion::cursor::Goto(1, 1),
                                termion::clear::CurrentLine,
                                field.mine_count,
                                field.flag_count
                            );
                            write!(
                                stdout,
                                "{}You lost!{}\r\n",
                                color::Fg(color::LightRed),
                                color::Fg(color::Reset)
                            );
                            break;
                        }
                    }
                    CellContent::Empty => {}
                },
                Key::Char('f') | Key::Char('F') => field.toggle_flag_at_cursor(),
                _ => {}
            }
        }
        write!(
            stdout,
            "{}{}Mines:{}    Flags:{}\r\n{field}",
            termion::cursor::Goto(1, 1),
            termion::clear::CurrentLine,
            field.mine_count,
            field.flag_count
        );
        if field.covered_empty_cells == 0 {
            write!(
                stdout,
                "{}You won!{}\r\n",
                color::Fg(color::Green),
                color::Fg(color::Reset)
            );
            stdout.flush().unwrap();
            break;
        }
        stdout.flush().unwrap();
    }
}
