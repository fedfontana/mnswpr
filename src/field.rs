use std::fmt::Display;

use rand::{thread_rng, Rng};
use termion::color;

use crate::cell;

pub struct Field {
    pub rows: usize,
    pub cols: usize,
    grid: Vec<cell::Cell>,
    pub covered_empty_cells: usize,
    pub mine_count: usize,
    pub flag_count: usize,
}

impl Field {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            grid: vec![cell::Cell::default(); rows * cols],
            covered_empty_cells: rows * cols,
            mine_count: 0,
            flag_count: 0,
        }
    }

    /// Returns the tuple (row, col) corresponding to the index passed as input
    fn idx_to_position(&self, idx: usize) -> (usize, usize) {
        let row = idx / self.cols;
        let col = idx % self.cols;
        (row, col)
    }

    fn position_to_idx(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    /// Randomizes the content of the field keeping a safe area of 1 tile around the cursor
    pub fn randomize(&mut self, mut mine_percentage: u8, current_row: usize, current_col: usize) {
        if mine_percentage >= 100 {
            mine_percentage = 99;
        }

        let mut covered_empty_cells = 0;
        let mut mine_count = 0;

        // Generate random board
        let mut rng = thread_rng();
        for idx in 0..self.rows * self.cols {
            let (row, col) = self.idx_to_position(idx);
            let cell_content = if row.abs_diff(current_row) >= 1 && col.abs_diff(current_col) >= 1 && rng.gen_range(1..=100) <= mine_percentage {
                mine_count += 1;
                cell::Content::Mine
            } else {
                covered_empty_cells += 1;
                cell::Content::Empty
            };

            let cell = cell::Cell {
                state: cell::State::Closed,
                content: cell_content,
                neighbouring_bomb_count: 0,
            };

            self.grid[idx] = cell;
        }

        self.covered_empty_cells = covered_empty_cells;
        self.mine_count = mine_count;
        self.recompute_neighbouroing_counts();
    }

    fn recompute_neighbouroing_counts(&mut self) {
        // Update the counters of the neighbouring mines for each mine
        for idx in 0..self.rows * self.cols {
            let mut count = 0;
            let row = (idx / self.cols) as isize;
            let col = (idx % self.cols) as isize;
            for delta_row in -1..=1 {
                for delta_col in -1..=1 {
                    // Do not count the current cell
                    if delta_col == 0 && delta_row == 0 {
                        continue;
                    }

                    let current_row = row + delta_row;
                    let current_col = col + delta_col;
                    // Do not consider out of bounds cells
                    if current_row >= self.rows as isize
                        || current_col >= self.cols as isize
                        || current_row < 0
                        || current_col < 0
                    {
                        continue;
                    }

                    match self.grid[current_row as usize * self.cols + current_col as usize].content
                    {
                        cell::Content::Mine => count += 1,
                        cell::Content::Empty => {}
                    }
                }
            }
            self.grid[row as usize * self.cols + col as usize].neighbouring_bomb_count = count;
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<cell::Cell> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        Some(self.grid[row * self.cols + col])
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut cell::Cell> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        Some(&mut self.grid[row * self.cols + col])
    }

    pub fn set(&mut self, row: usize, col: usize, new_value: cell::Cell) -> Option<cell::Cell> {
        let old_val = self.get(row, col);
        if old_val.is_none() {
            return None;
        }
        self.grid[row * self.cols + col] = new_value;
        Some(old_val.unwrap())
    }

    fn uncover_rec(field: &mut Field, current_row: isize, current_col: isize) {
        if current_row < 0 || current_col < 0 {
            return;
        }
        let res = field.get(current_row as usize, current_col as usize);
        if res.is_none() {
            return;
        }
        let current_cell = res.unwrap();

        // if state is Open or Flagged, do nothing
        if !matches!(current_cell.state, cell::State::Closed) {
            return;
        }

        if let cell::Content::Mine = current_cell.content {
            return;
        }

        field.covered_empty_cells -= 1;
        let current_cell = field
            .get_mut(current_row as usize, current_col as usize)
            .unwrap();
        current_cell.set_state(cell::State::Open);

        // Do not call recursively if we are at the edge of the 0s region
        if current_cell.neighbouring_bomb_count == 0 {
            // for each neighbouring cell run the function recursively
            for drow in -1..=1 {
                for dcol in -1..=1 {
                    if drow == 0 && dcol == 0 {
                        continue;
                    }
                    // Bounds will be checked by the next recursive call's `if let Some(..)` and the `if` before that
                    Self::uncover_rec(field, current_row + drow, current_col + dcol)
                }
            }
        }
    }

    pub fn uncover_at(&mut self, row: usize, col: usize) -> cell::Content {
        let old_content = self.get_mut(row, col).unwrap().content.clone();

        Self::uncover_rec(self, row as isize, col as isize);

        old_content
    }

    pub fn toggle_flag_at(&mut self, row: usize, col: usize) {
        let can_flag = self.flag_count < self.mine_count;
        let cell_under_cursor = self.get_mut(row, col).unwrap();
        match cell_under_cursor.state {
            cell::State::Open => {}
            cell::State::Closed => {
                if can_flag {
                    cell_under_cursor.set_state(cell::State::Flagged);
                    self.flag_count += 1;
                }
            }
            cell::State::Flagged => {
                cell_under_cursor.set_state(cell::State::Closed);
                self.flag_count -= 1;
            }
        };
    }

    pub fn uncover_all(&mut self) {
        self.grid
            .iter_mut()
            .for_each(|cell| cell.state = cell::State::Open);
    }
}