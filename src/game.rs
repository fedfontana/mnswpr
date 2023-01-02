use std::{fmt::Display, io::Write};

use crate::colors::{Palette, BG_RESET, DEFAULT_PALETTE, FG_RESET};
use crate::field::Field;
use termion::color;

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
}

impl Minesweeper {
    pub fn new(rows: usize, cols: usize, mine_percentage: u8) -> Self {
        Self {
            cursor: Cursor { row: 0, col: 0 },
            field: Field::new(rows, cols),
            rows,
            cols,
            mine_percentage,
        }
    }

    pub fn randomize_field(&mut self) {
        self.field
            .randomize(self.mine_percentage, self.cursor.row, self.cursor.col);
    }

    pub fn print_field(&self, f: &mut impl Write, palette: &Palette) {
        let mut str_repr = String::new(); //TODO use with_capacity()? / implement this in a more efficient way

        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = self.field.get(row, col).unwrap();

                let sep = if self.cursor.row == row && self.cursor.col == col {
                    ('[', ']')
                } else {
                    (' ', ' ')
                };

                str_repr = format!(
                    "{str_repr}{bg}{}{cell_repr}{bg}{}{BG_RESET}",
                    sep.0,
                    sep.1,
                    bg = palette.bg,
                    cell_repr = cell.to_string_with_palette(palette),
                );
            }
            str_repr = format!("{str_repr}\r\n");
        }

        write!(f, "{}", str_repr);
    }

    pub fn print_field_game_lost(&self, f: &mut impl Write, palette: &Palette) {
        let mut str_repr = String::new(); //TODO use with_capacity()? / implement this in a more efficient way

        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = self.field.get(row, col).unwrap();

                let sep = if self.cursor.row == row && self.cursor.col == col {
                    ('[', ']')
                } else {
                    (' ', ' ')
                };

                str_repr = format!(
                    "{str_repr}{bg}{}{cell_repr}{bg}{}{BG_RESET}",
                    sep.0,
                    sep.1,
                    bg = palette.bg,
                    cell_repr = cell.to_string_with_palette_lost(palette),
                );
            }
            str_repr = format!("{str_repr}\r\n");
        }

        write!(f, "{}", str_repr);
    }

    /// Prints the game state. Prints bomb count and flag count and appends the output of `self.print_field` to it
    pub fn print_game_state(&self, f: &mut impl Write) {
        write!(
            f,
            "{}Mines:{}    Flags:{}\r\n",
            termion::cursor::Goto(1, 1),
            self.field.mine_count,
            self.field.flag_count
        );
        self.print_field(f, &DEFAULT_PALETTE);
        f.flush().unwrap();
    }

    pub fn lose_screen(&self, f: &mut impl Write) {
        write!(
            f,
            "{}Mines:{}    Flags:{}\r\n",
            termion::cursor::Goto(1, 1),
            self.field.mine_count,
            self.field.flag_count
        );
        self.print_field_game_lost(f, &DEFAULT_PALETTE);
        write!(
            f,
            "{}You lost!{}\r\n",
            color::Fg(color::LightRed),
            color::Fg(color::Reset)
        );
        f.flush().unwrap();
    }
}

// impl Display for Field {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut str_repr = String::new(); //TODO use with_capacity()? / implement this in a more efficient way

//         for row in 0..self.rows {
//             for col in 0..self.cols {
//                 let cell = self.get(row, col).unwrap();
//                 if self.cursor.row == row && self.cursor.col == col {
//                     str_repr = format!("{str_repr}{BG_COLOR}[{cell}{BG_COLOR}]{}", color::Bg(color::Reset));
//                 } else {
//                     str_repr = format!("{str_repr}{BG_COLOR} {cell}{BG_COLOR} {}", color::Bg(color::Reset));
//                 }
//             }
//             str_repr = format!("{str_repr}\r\n");
//         }

//         write!(f, "{}", str_repr)
//     }
// }
