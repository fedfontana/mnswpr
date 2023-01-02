use std::fmt::Display;
use std::io::{stdin, stdout, Write};
use std::str::FromStr;

use clap::{command, Parser};

use termion::color;
use termion::cursor::HideCursor;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod cell;
mod colors;
mod field;
mod game;

use crate::game::Minesweeper;

#[derive(Clone)]
enum SizePreset {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
}

impl SizePreset {
    /// Returns the pair (columns, rows) corresponding the preset
    fn to_size(&self) -> (u64, u64) {
        match self {
            SizePreset::Tiny => (20, 13),
            SizePreset::Small => (30, 20),
            SizePreset::Medium => (40, 25),
            SizePreset::Large => (50, 30),
            SizePreset::Huge => (60, 40),
        }
    }
}

impl Display for SizePreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SizePreset::Tiny => "tiny",
            SizePreset::Small => "small",
            SizePreset::Medium => "medium",
            SizePreset::Large => "large",
            SizePreset::Huge => "huge",
        };
        write!(f, "{name}")
    }
}

impl FromStr for SizePreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "tiny" => Ok(SizePreset::Tiny),
            "small" => Ok(SizePreset::Small),
            "medium" => Ok(SizePreset::Medium),
            "large" => Ok(SizePreset::Large),
            "huge" => Ok(SizePreset::Huge),
            v => Err(format!(
                "Expected one of \"tiny\", \"small\", \"medium\", \"large\", \"huge\". Got \"{v}\""
            )),
        }
    }
}

/// A simple minesweeper game for the terminal.
///
/// Move the cursor with either wasd, hjkl or the arrows.
///
/// Flag/unflag the cell under the cursor by pressing f, or uncover it by pressing <space> or <insert>.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// The number of columns of the field. Must be greater than 1.
    #[arg(short, long="columns", value_parser=clap::value_parser!(u64).range(1..))]
    cols: Option<u64>,

    /// The number of rows of the field. Must be greater than 1.
    #[arg(short, long, value_parser=clap::value_parser!(u64).range(1..))]
    rows: Option<u64>,

    /// The percentage of mines in the field. Must be in the range (1, 100).
    #[arg(short, long, default_value_t=20, value_parser=clap::value_parser!(u8).range(1..100))]
    mine_percentage: u8,

    /// The size preset of the field. Note that `-c` and `-r` take precendence over the preset.
    #[arg(short, long, default_value_t=SizePreset::Tiny)]
    preset: SizePreset,
}

/// Returns (cols, rows) after parsing the cli arguments and clipping them with the size of the terminal minus some chars for padding
fn parse_field_size(args: &Args) -> (usize, usize) {
    let termsize = termion::terminal_size().unwrap();

    let cols = if args.cols.is_some() {
        args.cols.unwrap()
    } else {
        args.preset.to_size().0
    };
    // -2 is to have a little bit of padding, division by 3 is because we need to have enough space to print 3 chars for each tile
    let cols = (cols).min((termsize.0 as u64 - 2) / 3) as usize;

    let rows = if args.rows.is_some() {
        args.rows.unwrap()
    } else {
        args.preset.to_size().1
    };
    let rows = rows.min(termsize.1 as u64 - 4) as usize;

    (cols, rows)
}

fn main() {
    let args = Args::parse();

    let (cols, rows) = parse_field_size(&args);

    let mut game = Minesweeper::new(rows, cols, args.mine_percentage);

    let stdin = stdin();
    let mut stdout = HideCursor::from(stdout().into_raw_mode().unwrap());

    write!(
        stdout,
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1)
    ).unwrap();

    game.print_game_state(&mut stdout);
    stdout.flush().unwrap();

    let mut lost = false;
    let mut ask_play_again = false;
    let mut first_move = true;

    for c in stdin.events() {
        if let Event::Key(event) = c.unwrap() {
            match event {
                Key::Char(' ') | Key::Char('y') | Key::Char('Y') | Key::Insert
                    if ask_play_again =>
                {
                    lost = false;
                    ask_play_again = false;
                    first_move = true;

                    write!(
                        stdout,
                        "{}{}",
                        termion::clear::All,
                        termion::cursor::Goto(1, 1)
                    ).unwrap();
                    game.reset();
                }
                Key::Char('q') | Key::Char('Q') | Key::Char('n') | Key::Char('N')
                    if ask_play_again =>
                {
                    break
                }
                Key::Char('q') | Key::Char('Q') => break,
                Key::Char('w') | Key::Char('W') | Key::Char('k') | Key::Char('K') | Key::Up if !ask_play_again => {
                    if game.cursor.row > 0 {
                        game.cursor.row -= 1;
                    }
                }
                Key::Char('a') | Key::Char('A') | Key::Char('h') | Key::Char('H') | Key::Left if !ask_play_again => {
                    if game.cursor.col > 0 {
                        game.cursor.col -= 1;
                    }
                }
                Key::Char('s') | Key::Char('S') | Key::Char('j') | Key::Char('J') | Key::Down if !ask_play_again => {
                    if game.cursor.row < game.rows - 1 {
                        game.cursor.row += 1;
                    }
                }
                Key::Char('d') | Key::Char('D') | Key::Char('l') | Key::Char('L') | Key::Right if !ask_play_again => {
                    if game.cursor.col < game.cols - 1 {
                        game.cursor.col += 1;
                    }
                }
                Key::Char(' ') | Key::Insert if !ask_play_again => {
                    if first_move {
                        game.randomize_field();
                        first_move = false;
                    }

                    if game.field.uncover_at(game.cursor.row, game.cursor.col) {
                        game.lose_screen(&mut stdout);
                        write!(stdout, "Press y/Y/<space>/<insert> if you want to play again, otherwise press n/N\r\n").unwrap();
                        lost = true;
                        ask_play_again = true;
                    }
                }
                Key::Char('f') | Key::Char('F') if !first_move && !ask_play_again => {
                    game.field.toggle_flag_at(game.cursor.row, game.cursor.col)
                }
                _ => {}
            }
        }
        if !lost {
            game.print_game_state(&mut stdout);
            if game.field.covered_empty_cells == 0 {
                write!(
                    &mut stdout,
                    "{}You won!{}\r\n",
                    color::Fg(color::Green),
                    color::Fg(color::Reset)
                ).unwrap();
                stdout.flush().unwrap();
                write!(
                    stdout,
                    "Do you want to play again? Press y/Y/<space>/<insert> if yes, n/N if no\r\n"
                ).unwrap();
                ask_play_again = true;
            }
        }
    }
    stdout.flush().unwrap();
}
