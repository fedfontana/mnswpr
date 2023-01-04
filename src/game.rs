use std::io::Write;
use std::ops::{Deref, DerefMut};

use termion::color;

use crate::colors::{Palette, BG_RESET, FG_RESET};
use crate::field::Field;

pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

pub enum Direction {
    Up,
    Left,
    Right,
    Down,
}

pub struct Mnswpr {
    pub field: Field,
    pub cursor: Cursor,
    pub rows: usize,
    pub cols: usize,
    mine_percentage: u8,
    palette: Palette,
}

impl Mnswpr {
    pub fn new(rows: usize, cols: usize, mine_percentage: u8, palette: Palette) -> Self {
        Self {
            cursor: Cursor { row: 0, col: 0 },
            field: Field::new(rows, cols),
            rows,
            cols,
            mine_percentage,
            palette,
        }
    }

    /// Randomizes the field keeping a safe area around the current position of the cursor
    pub fn randomize_field(&mut self) {
        self.field
            .randomize(self.mine_percentage, self.cursor.row, self.cursor.col);
    }

    /// Prints the field with the current palette.
    /// May return an error if it was not able to write in `f`
    pub fn print_field(&self, f: &mut impl Write) -> anyhow::Result<()> {
        let mut str_repr = String::with_capacity(self.rows * self.cols * 3 * 2);

        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = self.field.get_unchecked(row, col);

                let cell_repr = cell.to_string_with_palette(
                    &self.palette,
                    self.cursor.row == row && self.cursor.col == col,
                );
                str_repr.push_str(&cell_repr);
            }
            str_repr = format!("{str_repr}{BG_RESET}{FG_RESET}\r\n");
        }

        write!(f, "{str_repr}")?;
        Ok(())
    }

    /// Prints the field with the current palette, highlighting the flags that were placed in the right
    /// place and the ones that are placed on empty cells
    /// May return an error if it was not able to write in `f`
    pub fn print_field_game_lost(&self, f: &mut impl Write) -> anyhow::Result<()> {
        let mut str_repr = String::with_capacity(self.rows * self.cols * 3 * 2);

        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = self.field.get_unchecked(row, col);

                let cell_repr = cell.to_string_with_palette_lost(
                    &self.palette,
                    self.cursor.row == row && self.cursor.col == col,
                );
                str_repr.push_str(&cell_repr);
            }
            str_repr = format!("{str_repr}{BG_RESET}{FG_RESET}\r\n");
        }

        write!(f, "{str_repr}")?;
        Ok(())
    }

    /// Prints the game state. Prints bomb count and flag count and appends 
    /// the output of `self.print_field` to it
    /// May return an error if it was not able to write in `f`
    pub fn print_game_state(&self, f: &mut impl Write) -> anyhow::Result<()> {
        write!(
            f,
            "{}Mines:{}    Flags:{}\r\n",
            termion::cursor::Goto(1, 1),
            self.field.mine_count,
            self.field.flag_count
        )?;
        self.print_field(f)?;
        f.flush()?;
        Ok(())
    }

    /// Prints the bomb count, flag count, appends the result of `self.print_field_game_lost` 
    /// to it and then prints `You lost`
    /// May return an error if it was not able to write in `f`
    pub fn lose_screen(&self, f: &mut impl Write) -> anyhow::Result<()> {
        write!(
            f,
            "{}Mines:{}    Flags:{}\r\n",
            termion::cursor::Goto(1, 1),
            self.field.mine_count,
            self.field.flag_count
        )?;
        self.print_field_game_lost(f)?;
        write!(
            f,
            "{}You lost!{}\r\n",
            color::Fg(color::LightRed),
            color::Fg(color::Reset)
        )?;
        f.flush()?;
        Ok(())
    }

    pub fn move_cursor(&mut self, d: Direction) {
        match d {
            Direction::Up => {
                if self.cursor.row > 0 {
                    self.cursor.row -= 1;
                }
            },
            Direction::Left => {
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                }
            },
            Direction::Right => {
                if self.cursor.col < self.cols - 1 {
                    self.cursor.col += 1;
                }
            },
            Direction::Down => {
                if self.cursor.row < self.rows - 1 {
                    self.cursor.row += 1;
                }
            },
        }
    }
}

impl Deref for Mnswpr {
    type Target = Field;

    fn deref(&self) -> &Self::Target {
        &self.field
    }
}

impl DerefMut for Mnswpr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.field
    }
}