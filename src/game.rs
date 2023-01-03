use std::io::Write;

use termion::color;

use crate::colors::{Palette, BG_RESET, FG_RESET};
use crate::field::Field;

pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

pub struct Minesweeper {
    pub field: Field,
    pub cursor: Cursor,
    pub rows: usize,
    pub cols: usize,
    mine_percentage: u8,
    palette: Palette
}

impl Minesweeper {
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

    pub fn reset(&mut self) {
        self.field.reset();
    }

    pub fn randomize_field(&mut self) {
        self.field
            .randomize(self.mine_percentage, self.cursor.row, self.cursor.col);
    }

    pub fn print_field(&self, f: &mut impl Write) {
        let mut str_repr = String::with_capacity(self.rows * self.cols * 3 * 2);

        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = self.field.get(row, col).unwrap();

                let cell_repr = cell.to_string_with_palette(
                    &self.palette,
                    self.cursor.row == row && self.cursor.col == col,
                );
                str_repr.push_str(&cell_repr);
            }
            str_repr = format!("{str_repr}{FG_RESET}{BG_RESET}\r\n");
        }

        write!(f, "{str_repr}").unwrap();
    }

    pub fn print_field_game_lost(&self, f: &mut impl Write) {
        let mut str_repr = String::with_capacity(self.rows * self.cols * 3 * 2);

        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = self.field.get(row, col).unwrap();

                let cell_repr = cell.to_string_with_palette_lost(
                    &self.palette,
                    self.cursor.row == row && self.cursor.col == col,
                );
                str_repr.push_str(&cell_repr);
            }
            str_repr = format!("{str_repr}{BG_RESET}{FG_RESET}\r\n");
        }

        write!(f, "{str_repr}").unwrap();
    }

    /// Prints the game state. Prints bomb count and flag count and appends the output of `self.print_field` to it
    pub fn print_game_state(&self, f: &mut impl Write) {
        write!(
            f,
            "{}Mines:{}    Flags:{}\r\n",
            termion::cursor::Goto(1, 1),
            self.field.mine_count,
            self.field.flag_count
        )
        .unwrap();
        self.print_field(f);
        f.flush().unwrap();
    }

    pub fn lose_screen(&self, f: &mut impl Write) {
        write!(
            f,
            "{}Mines:{}    Flags:{}\r\n",
            termion::cursor::Goto(1, 1),
            self.field.mine_count,
            self.field.flag_count
        )
        .unwrap();
        self.print_field_game_lost(f);
        write!(
            f,
            "{}You lost!{}\r\n",
            color::Fg(color::LightRed),
            color::Fg(color::Reset)
        )
        .unwrap();
        f.flush().unwrap();
    }
}
