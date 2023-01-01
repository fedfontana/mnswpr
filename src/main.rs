#![allow(dead_code, unused)]

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::fmt::Display;

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

impl Default for CellContent {
    fn default() -> Self {
        Self::Empty
    }
}

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
                CellContent::Mine => write!(f, "*"),
                CellContent::Empty => write!(f, "{}", self.neighboring_bomb_count),
            },
            CellState::Closed => write!(f, "."),
            CellState::Flagged => write!(f, "F"),
        }
    }
}

struct Field {
    rows: usize,
    cols: usize,
    grid: Vec<Cell>,
}

impl Field {
    fn new(rows: usize, cols: usize, mut bomb_percentage: usize) -> Self {
        if bomb_percentage > 100 {
            bomb_percentage = 100;
        }

        let mut grid = Vec::with_capacity(rows * cols);

        // Generate random board
        let mut rng = thread_rng();
        for idx in 0..rows * cols {
            let cell_content = if rng.gen_range(1..=100) <= bomb_percentage {
                CellContent::Mine
            } else {
                CellContent::Empty
            };

            let cell = Cell {
                state: CellState::Open,
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

        Self { rows, cols, grid }
    }

    fn get(&self, row: usize, col: usize) -> Option<Cell> {
        if row > self.rows || col > self.cols {
            return None;
        }
        Some(self.grid[row * self.cols + col])
    }

    fn set(&mut self, row: usize, col: usize, new_value: Cell) -> Option<Cell> {
        let old_val = self.get(row, col);
        if old_val.is_none() {
            return None;
        }
        self.grid[row * self.cols + col] = new_value;
        Some(old_val.unwrap())
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut str_repr = "".to_string();

        for row in 0..self.rows {
            let new_row_content = self
                .grid
                .iter()
                .skip(row * self.cols)
                .take(self.cols)
                .map(|el| el.to_string())
                .collect::<Vec<_>>()
                .join("  ");
            str_repr = format!("{}\n{}\n", str_repr, new_row_content);
        }

        write!(f, "{}", str_repr)
    }
}

fn main() {
    let mut field = Field::new(10, 10, 20);
    println!("{field}");
}
