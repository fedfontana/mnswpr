use termion::cursor::HideCursor;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::RawTerminal;

use std::io::{stdin, Stdout, Write};
use std::ops::{Deref, DerefMut};

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
                let cell = self.get_unchecked(row, col);

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
                let cell = self.get_unchecked(row, col);

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

    /// Prints the game state. Prints header (bomb and flag count)
    /// Then prints the current state of the board if `open_everything == false`
    /// otherwise print the open board with the status of the flags (placed correctly on a mine or placed on an empty cell)
    /// May return an error if it was not able to write in `f`
    pub fn print_game_state(
        &self,
        f: &mut impl Write,
        open_everything: bool,
    ) -> anyhow::Result<()> {
        write!(
            f,
            "{}Mines:{}    Flags:{}\r\n",
            termion::cursor::Goto(1, 1),
            self.field.mine_count,
            self.field.flag_count
        )?;
        if !open_everything {
            self.print_field(f)?;
        } else {
            self.print_field_game_lost(f)?;
        }
        f.flush()?;
        Ok(())
    }

    pub fn move_cursor(&mut self, d: Direction) {
        match d {
            Direction::Up => {
                if self.cursor.row > 0 {
                    self.cursor.row -= 1;
                }
            }
            Direction::Left => {
                if self.cursor.col > 0 {
                    self.cursor.col -= 1;
                }
            }
            Direction::Right => {
                if self.cursor.col < self.cols - 1 {
                    self.cursor.col += 1;
                }
            }
            Direction::Down => {
                if self.cursor.row < self.rows - 1 {
                    self.cursor.row += 1;
                }
            }
        }
    }

    /// Handles the game loop for one game
    /// Returns Some(true) if the user won the current game
    /// Returns Some(false) if the user lost the current game
    /// Returns None if the user quit
    pub fn play(
        &mut self,
        stdout: &mut HideCursor<RawTerminal<Stdout>>,
        assisted_opening: bool,
        assisted_flagging: bool,
    ) -> anyhow::Result<Option<bool>> {
        let stdin = stdin();

        self.print_game_state(stdout, false)?;
        stdout.flush()?;

        let mut first_move = true;

        for e in stdin.events() {
            let e = e?;
            if !matches!(e, Event::Key(_)) {
                continue;
            }
            let Event::Key(event) = e else { unreachable!(); };

            let crow = self.cursor.row;
            let ccol = self.cursor.col;

            match event {
                Key::Char('q' | 'Q') => return Ok(None),
                Key::Char('w' | 'W' | 'k' | 'K') | Key::Up => self.move_cursor(Direction::Up),
                Key::Char('a' | 'A' | 'h' | 'H') | Key::Left => self.move_cursor(Direction::Left),
                Key::Char('s' | 'S' | 'j' | 'J') | Key::Down => self.move_cursor(Direction::Down),
                Key::Char('d' | 'D' | 'l' | 'L') | Key::Right => self.move_cursor(Direction::Right),
                Key::Char(' ' | '\n') => {
                    if first_move {
                        self.randomize_field();
                        first_move = false;
                    }

                    let cell = self.get_unchecked(crow, ccol);

                    if assisted_opening
                        && cell.is_open()
                        && self
                            .get_flagged_nbors_amt(crow, ccol)
                            .expect("Cursor out of bounds")
                            == cell.neighbouring_bomb_count
                    {
                        let exploded = self
                            .uncover_around_cell_at(crow, ccol)
                            .expect("Cursor out of bounds");
                        if exploded {
                            return Ok(Some(false));
                        }
                    } else {
                        if self
                            .uncover_at(crow, ccol)
                            .expect("Cursor out of bounds")
                        {
                            return Ok(Some(false));
                        }
                    }
                }
                Key::Char('f' | 'F') if !first_move => {
                    if assisted_flagging {
                        let cell = self.get_unchecked(crow, ccol);

                        let non_open_nbors = self
                            .get_non_open_nbors_amt(crow, ccol)
                            .expect("Position out of bounds");

                        if cell.is_open() && cell.neighbouring_bomb_count == non_open_nbors {
                            self.unflag_all_closed_around(crow, ccol);
                        }
                    }

                    self.toggle_flag_at(crow, ccol)
                        .expect("Cursor position out of bounds");
                }
                _ => {}
            }
            self.print_game_state(stdout, false)?;
            if self.field.closed_empty_cells == 0 {
                return Ok(Some(true));
            }
        }
        Ok(None)
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
