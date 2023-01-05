use rand::{thread_rng, Rng};

use crate::cell;

pub struct Field {
    pub rows: usize,
    pub cols: usize,
    grid: Vec<cell::Cell>,
    pub closed_empty_cells: usize,
    pub mine_count: usize,
    pub flag_count: usize,
}

impl Field {
    /// Constructs a new `Field`.
    /// Note that if `rows == 0`, it gets set to 1. Same with `cols`.
    pub fn new(rows: usize, cols: usize) -> Self {
        let rows = rows.max(1);
        let cols = cols.max(1);

        Self {
            rows,
            cols,
            grid: vec![cell::Cell::default(); rows * cols],
            closed_empty_cells: rows * cols,
            mine_count: 0,
            flag_count: 0,
        }
    }

    /// Resets the field state to an empty field with the same rows and cols
    pub fn reset(&mut self) {
        self.grid = vec![cell::Cell::default(); self.rows * self.cols];
        self.closed_empty_cells = self.rows * self.cols;
        self.mine_count = 0;
        self.flag_count = 0;
    }

    /// Returns the tuple (row, col) corresponding to the index passed as input
    /// Does not check whether the resulting position is out of bounds
    fn idx_to_position(&self, idx: usize) -> (usize, usize) {
        let row = idx / self.cols;
        let col = idx % self.cols;
        (row, col)
    }

    /// Returns the index corresponding to the position (row, col) passed as input
    /// Does not check whether the resultin position is out of bounds
    fn position_to_idx(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    /// Randomizes the content of the field keeping a safe area of 1 tile around the cursor
    pub fn randomize(&mut self, mut mine_percentage: u8, current_row: usize, current_col: usize) {
        if mine_percentage >= 100 {
            mine_percentage = 99;
        }

        let mut closed_empty_cells = 0;
        let mut mine_count = 0;

        // Generate random board
        let mut rng = thread_rng();
        for idx in 0..self.rows * self.cols {
            let (row, col) = self.idx_to_position(idx);

            let cell_content = if rng.gen_range(1..=100) <= mine_percentage {
                if row.abs_diff(current_row) < 2 && col.abs_diff(current_col) < 2 {
                    closed_empty_cells += 1;
                    cell::Content::Empty
                } else {
                    mine_count += 1;
                    cell::Content::Mine
                }
            } else {
                closed_empty_cells += 1;
                cell::Content::Empty
            };

            self.grid[idx] = cell::Cell {
                state: cell::State::Closed,
                content: cell_content,
                neighbouring_bomb_count: 0,
            };
        }

        self.closed_empty_cells = closed_empty_cells;
        self.mine_count = mine_count;

        self.recompute_neighbouroing_counts();
    }

    /// Updates the neighboring bomb count for each cell in the field.
    fn recompute_neighbouroing_counts(&mut self) {
        for idx in 0..self.rows * self.cols {
            let mut count = 0;
            let (row, col) = self.idx_to_position(idx);

            for delta_row in -1..=1 {
                for delta_col in -1..=1 {
                    // Do not count the current cell
                    if delta_col == 0 && delta_row == 0 {
                        continue;
                    }

                    let current_row = row as isize + delta_row;
                    let current_col = col as isize + delta_col;

                    // Do not consider out of bounds cells
                    if current_row >= self.rows as isize
                        || current_col >= self.cols as isize
                        || current_row < 0
                        || current_col < 0
                    {
                        continue;
                    }

                    if self
                        .get_unchecked(current_row as usize, current_col as usize)
                        .contains_mine()
                    {
                        count += 1;
                    }
                }
            }
            self.grid[idx].neighbouring_bomb_count = count;
        }
    }

    /// Returns a reference to the cell at position (row, col).
    /// Returns None if the position (row, col) is out of bounds
    pub fn get(&self, row: usize, col: usize) -> Option<&cell::Cell> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        Some(&self.grid[self.position_to_idx(row, col)])
    }

    /// Returns a reference to the cell at position (row, col).
    /// Note that this method does not check for boundaries, and as such it may panic
    pub fn get_unchecked(&self, row: usize, col: usize) -> &cell::Cell {
        &self.grid[self.position_to_idx(row, col)]
    }

    /// Returns an exclusive reference to the cell at position (row, col).
    /// Returns None if the position (row, col) is out of bounds
    pub fn get_mut(&mut self, row: usize, col: usize) -> Option<&mut cell::Cell> {
        if row >= self.rows || col >= self.cols {
            return None;
        }
        let idx = self.position_to_idx(row, col);
        Some(&mut self.grid[idx])
    }

    /// Returns an exclusive reference to the cell at position (row, col).
    /// Note that this method does not check for boundaries, and as such it may panic
    pub fn get_mut_unchecked(&mut self, row: usize, col: usize) -> &mut cell::Cell {
        let idx = self.position_to_idx(row, col);
        &mut self.grid[idx]
    }

    fn uncover_rec(field: &mut Field, current_row: isize, current_col: isize) {
        if current_row < 0 || current_col < 0 {
            return;
        }

        let opt_cell = field.get(current_row as usize, current_col as usize);

        // Do not go out of bounds
        if opt_cell.is_none() {
            return;
        }

        let cell = opt_cell.unwrap();

        // if state is Open or Flagged, do nothing
        if !cell.is_closed() {
            return;
        }

        //TODO I think this is useless and wrong, just leaving it here until i make up my mind about it
        // Stop the recursion when meeting a mine
        // if !cell.is_empty() {
        //     return;
        // }

        field.closed_empty_cells -= 1;

        let current_cell = field.get_mut_unchecked(current_row as usize, current_col as usize);
        current_cell.set_state(cell::State::Open);

        // Do not call recursively if we are at the edge of the 0s region
        if current_cell.neighbouring_bomb_count != 0 {
            return;
        }

        // for each neighbouring cell run the function recursively
        for drow in -1..=1 {
            for dcol in -1..=1 {
                if drow == 0 && dcol == 0 {
                    continue;
                }

                // Bounds will be checked by the recursive call
                Self::uncover_rec(field, current_row + drow, current_col + dcol)
            }
        }
    }

    /// Uncovers the board recursively (when meeting non-mine tiles with 0 neighbouring mines)
    /// starting at position (row, col). Returns whether the selected cell contained an un-flagged mine
    /// Return None if (row, col) is out of bounds
    pub fn uncover_at(&mut self, row: usize, col: usize) -> Option<bool> {
        let old_cell = self.get_mut(row, col)?;

        // Return early if the user tried to open a mine
        if old_cell.contains_mine() && !old_cell.is_flagged() {
            return Some(true);
        }

        Self::uncover_rec(self, row as isize, col as isize);

        Some(false)
    }

    /// Toggles the state of the cell at position (row, col) and updates `self.flag_count`
    /// Returns None if position (row, col) is out of bounds
    pub fn toggle_flag_at(&mut self, row: usize, col: usize) -> Option<()> {
        let can_flag = self.flag_count < self.mine_count;

        let cell_under_cursor = self.get_mut(row, col)?;

        match cell_under_cursor.state {
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
            _ => {}
        };

        Some(())
    }

    /// Returns the number of flagged neighbors of the cell at position (row, col).
    /// Returns None if the position is out of bounds
    pub fn get_flagged_nbors_amt(&self, row: usize, col: usize) -> Option<usize> {
        self.get_nbor_amt_that_match(row, col, |c| c.is_flagged())
    }

    /// Uncovers the closed cells around the cell at (row, col).
    /// Returns None if the position (row, col) is out of bounds, otherwise
    /// it returns Some(true) if there was a mine around the current cell (returns as soon as a mine is found),
    /// otherwise returns Some(false).
    pub fn uncover_around_cell_at(&mut self, row: usize, col: usize) -> Option<bool> {
        // Return early None if (row, col) is out of bounds
        self.get(row, col)?;

        let row = row as isize;
        let col = col as isize;

        for drow in -1..=1 {
            for dcol in -1..=1 {
                if drow == 0 && dcol == 0 {
                    continue;
                }
                if row + drow < 0 || col + dcol < 0 {
                    continue;
                }

                let r = (row + drow) as usize;
                let c = (col + dcol) as usize;

                if let Some(cell) = self.get(r, c) {
                    if cell.is_closed() {
                        if self.uncover_at(r, c).expect("Index out of bounds") {
                            return Some(true);
                        }
                    }
                }
            }
        }
        Some(false)
    }

    /// Returns the number of closed or flagged neighbors of the cell at position (row, col).
    /// Returns None if the position is out of bounds
    pub fn get_non_open_nbors_amt(&self, row: usize, col: usize) -> Option<usize> {
        self.get_nbor_amt_that_match(row, col, |c| !c.is_open())
    }

    /// Flags the closed cells around the cell at (row, col).
    /// Returns false if the position (row, col) is out of bounds, returns true otherwise
    pub fn unflag_all_closed_around(&mut self, row: usize, col: usize) -> bool {
        if self.get(row, col).is_none() {
            return false;
        }

        let row = row as isize;
        let col = col as isize;

        for drow in -1..=1 {
            for dcol in -1..=1 {
                if drow == 0 && dcol == 0 {
                    continue;
                }
                if row + drow < 0 || col + dcol < 0 {
                    continue;
                }

                let r = (row + drow) as usize;
                let c = (col + dcol) as usize;

                if let Some(cell) = self.get(r, c) {
                    if cell.is_closed() {
                        self.toggle_flag_at(r, c);
                    }
                }
            }
        }

        true
    }

    /// Returns the number of neighbors of the cell at position (row, col) that make the predicate `match_fn` return true.
    /// Returns None if the position (row, col) is out of bounds
    fn get_nbor_amt_that_match(
        &self,
        row: usize,
        col: usize,
        match_fn: impl Fn(&cell::Cell) -> bool,
    ) -> Option<usize> {
        // Return early None if position (row, col) is out of bounds
        self.get(row, col)?;

        let mut count = 0;

        let row = row as isize;
        let col = col as isize;

        for drow in -1..=1 {
            for dcol in -1..=1 {
                if drow == 0 && dcol == 0 {
                    continue;
                }
                if row + drow < 0 || col + dcol < 0 {
                    continue;
                }
                let opt_cell = self.get((row + drow) as usize, (col + dcol) as usize);
                if let Some(cell) = opt_cell {
                    if match_fn(cell) {
                        count += 1;
                    }
                }
            }
        }
        Some(count)
    }
}
