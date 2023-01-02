#![allow(dead_code, unused)]

use std::fmt::Display;
use std::io::{stdin, stdout, Write};

use clap::{Parser, command};
use rand::{seq::SliceRandom, thread_rng, Rng};

use termion::color;
use termion::cursor::HideCursor;
use termion::event::{Event, Key, MouseEvent};
use termion::input::{MouseTerminal, TermRead};
use termion::raw::IntoRawMode;



mod field;
mod game;
mod colors;
mod cell;

use field::Field;
use crate::game::Minesweeper;

//TODO add mouse click support (supported by termion)?
//TODO add docs and tests
//TODO run under the strictest clippy
//TODO add cli options:
//  - widht
//  - height
//  - mine_percentage
//  - presets like easy, hard, medium, ...
//  - auto-counter or something like that. When this option is active, numbers that have too many flags around them turn bright red,
//          and the ones with the right amount of flags around them get printed green

//TODO add a retry option after concluding a match
//TODO add bg color for uncovered cells (the original gray (185, 185, 185))
//TODO generate board when clicking on first cell. 
//  - Either generate a number or a whole area of numbers under the cursor in a way that the first tile cannot be a bomb.

fn percentage_validator(v: usize) -> Result<(), String> {
    if v < 100 && v > 0 {
        Ok(())
    } else {
        Err("The value of --mine-percentage must be in the range (0,100)".to_string())
    }
}


/// A simple minesweeper game for the terminal. 
/// 
/// Move the cursor with either wasd, hjkl or the arrows.
/// 
/// Flag/unflag the cell under the cursor by pressing f, or uncover it by pressing the spacebar or enter.
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// The number of columns of the field. Must be greater than 1.
    #[arg(long="columns", default_value_t=30, value_parser=clap::value_parser!(u64).range(1..))]
    cols: u64,

    /// The number of rows of the field. Must be greater than 1.
    #[arg(long, default_value_t=20, value_parser=clap::value_parser!(u64).range(1..))]
    rows: u64,

    /// The percentage of mines in the field. Must be in the range (1, 100).
    #[arg(short, long, default_value_t=20, value_parser=clap::value_parser!(u8).range(1..100))]
    mine_percentage: u8,
}


fn main() {
    let args = Args::parse();

    let termsize = termion::terminal_size().unwrap();
    let cols = args.cols.min(termsize.0 as u64 - 2) as usize;
    let rows = args.rows.min(termsize.1 as u64 - 4) as usize;

    let mut game = Minesweeper::new(rows, cols, args.mine_percentage);
    game.randomize_field();

    let stdin = stdin();
    let mut stdout = HideCursor::from(stdout().into_raw_mode().unwrap());

    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1,1));

    game.print_game_state(&mut stdout);
    stdout.flush().unwrap();

    for c in stdin.events() {
        if let Event::Key(event) = c.unwrap() {
            match event {
                Key::Char('q') | Key::Char('Q') => break,
                Key::Char('w') | Key::Char('W') | Key::Char('k') | Key::Char('K') | Key::Up => {
                    if game.cursor.row > 0 {
                        game.cursor.row -= 1;
                    }
                }
                Key::Char('a') | Key::Char('A') |  Key::Char('h') | Key::Char('H') | Key::Left => {
                    if game.cursor.col > 0 {
                        game.cursor.col -= 1;
                    }
                }
                Key::Char('s') | Key::Char('S') |  Key::Char('j') | Key::Char('J') | Key::Down => {
                    if game.cursor.row < game.rows - 1 {
                        game.cursor.row += 1;
                    }
                }
                Key::Char('d') | Key::Char('D') |  Key::Char('l') | Key::Char('L') | Key::Right => {
                    if game.cursor.col < game.cols - 1 {
                        game.cursor.col += 1;
                    }
                }
                Key::Char(' ') => match game.field.uncover_at(game.cursor.row, game.cursor.col) {
                    cell::Content::Mine => {
                        let cell = game.field.get(game.cursor.row, game.cursor.col).unwrap();
                        if !matches!(cell.state, cell::State::Flagged) {
                            game.lose_screen(&mut stdout);
                            break;
                        }
                    }
                    cell::Content::Empty => {}
                },
                Key::Char('f') | Key::Char('F') => game.field.toggle_flag_at(game.cursor.row, game.cursor.col),
                _ => {}
            }
        }
        game.print_game_state(&mut stdout);
        if game.field.covered_empty_cells == 0 {
            write!(
                &mut stdout,
                "{}You won!{}\r\n",
                color::Fg(color::Green),
                color::Fg(color::Reset)
            );
            stdout.flush().unwrap();
            break;
        }
    }
    stdout.flush().unwrap();
}
