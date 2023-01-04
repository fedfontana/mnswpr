
use termion::color;
use termion::cursor::HideCursor;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::RawTerminal;



use std::io::{Write, Stdout, stdin};
use std::ops::{Deref, DerefMut};


use crate::cell;
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

    pub fn play(&mut self, stdout: &mut HideCursor<RawTerminal<Stdout>>, assisted_opening: bool, assisted_flagging: bool) -> anyhow::Result<bool> {
        let stdin = stdin();
        
        self.print_game_state(stdout)?;
        stdout.flush()?;

        let mut lost = false;
        let mut ask_play_again = false;
        let mut first_move = true;

        for c in stdin.events() {
            if let Ok(Event::Key(event)) = c {
                match event {
                    Key::Char(' ' | 'y' | 'Y' | '\n') if ask_play_again => {
                        lost = false;
                        ask_play_again = false;
                        first_move = true;

                        write!(
                            stdout,
                            "{}{}",
                            termion::clear::All,
                            termion::cursor::Goto(1, 1)
                        )?;
                        self.reset();
                    }
                    Key::Char('q' | 'Q' | 'n' | 'N') if ask_play_again => break,
                    Key::Char('q' | 'Q') => break,
                    Key::Char('w' | 'W' | 'k' | 'K') | Key::Up if !ask_play_again => {
                        self.move_cursor(Direction::Up)
                    }
                    Key::Char('a' | 'A' | 'h' | 'H') | Key::Left if !ask_play_again => {
                        self.move_cursor(Direction::Left)
                    }
                    Key::Char('s' | 'S' | 'j' | 'J') | Key::Down if !ask_play_again => {
                        self.move_cursor(Direction::Down)
                    }
                    Key::Char('d' | 'D' | 'l' | 'L') | Key::Right if !ask_play_again => {
                        self.move_cursor(Direction::Right)
                    }
                    Key::Char(' ' | '\n') if !ask_play_again => {
                        if first_move {
                            self.randomize_field();
                            first_move = false;
                        }

                        let cell = self.field.get_unchecked(self.cursor.row, self.cursor.col);
                        if assisted_opening
                            && matches!(cell.state, cell::State::Open)
                            && self
                                .field
                                .get_flagged_nbors_amt(self.cursor.row, self.cursor.col)
                                .expect("Position out of bounds")
                                == cell.neighbouring_bomb_count
                        {
                            if self
                                .field
                                .uncover_around_cell_at(self.cursor.row, self.cursor.col)
                                .expect("Position out of bounds")
                            {
                                self.lose_screen(stdout)?;
                                write!(stdout, "Press y/Y/<space>/<insert> if you want to play again, otherwise press n/N\r\n")?;
                                lost = true;
                                ask_play_again = true;
                            }
                        } else {
                            if self
                                .field
                                .uncover_at(self.cursor.row, self.cursor.col)
                                .expect("Cursor out of bounds")
                            {
                                self.lose_screen(stdout)?;
                                write!(stdout, "Press y/Y/<space>/<insert> if you want to play again, otherwise press n/N\r\n")?;
                                lost = true;
                                ask_play_again = true;
                            }
                        }
                    }
                    Key::Char('f' | 'F') if !first_move && !ask_play_again => {
                        if assisted_flagging {
                            let cell = self
                                .field
                                .get(self.cursor.row, self.cursor.col)
                                .expect("Cursor position out of bounds");

                            let non_open_nbors = self
                                .get_non_open_nbors_amt(self.cursor.row, self.cursor.col)
                                .expect("Position out of bounds");

                            if matches!(cell.state, cell::State::Open)
                                && cell.neighbouring_bomb_count == non_open_nbors
                            {
                                self.field
                                    .unflag_all_closed_around(self.cursor.row, self.cursor.col);
                            }
                        }

                        self.field
                            .toggle_flag_at(self.cursor.row, self.cursor.col)
                            .expect("Cursor position out of bounds");
                    }
                    _ => {}
                }
            }
            if !lost {
                self.print_game_state(stdout)?;
                if self.field.covered_empty_cells == 0 {
                    write!(
                        stdout,
                        "{}You won!{}\r\n",
                        color::Fg(color::Green),
                        color::Fg(color::Reset)
                    )?;
                    stdout.flush()?;
                    write!(
                    stdout,
                    "Do you want to play again? Press y/Y/<space>/<insert> if yes, n/N if no\r\n"
                )?;
                    ask_play_again = true;
                }
            }
        }
        Ok(false)
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
