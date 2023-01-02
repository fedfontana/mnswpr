#![allow(dead_code, unused)]

use std::fmt::Display;
use std::io::{stdin, stdout, Write};

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
//TODO add cli options:
//  - widht
//  - height
//  - mine_percentage
//  - presets like easy, hard, medium, ...
//  - auto-counter or something like that. When this option is active, numbers that have too many flags around them turn bright red,
//          and the ones with the right amount of flags around them get printed green

//TODO add a retry option after concluding a match
//TODO when a match is lost, highlight the flags in the wrong place with a different color
//TODO add bg color for uncovered cells (the original gray (185, 185, 185))
//TODO generate board when clicking on first cell. 
//  - Either generate a number or a whole area of numbers under the cursor in a way that the first tile cannot be a bomb.



fn main() {
    let mut game = Minesweeper::new(10, 10, 20);
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
                Key::Char('w') | Key::Char('W') | Key::Up => {
                    if game.cursor.row > 0 {
                        game.cursor.row -= 1;
                    }
                }
                Key::Char('a') | Key::Char('A') | Key::Left => {
                    if game.cursor.col > 0 {
                        game.cursor.col -= 1;
                    }
                }
                Key::Char('s') | Key::Char('S') | Key::Down => {
                    if game.cursor.row < game.rows - 1 {
                        game.cursor.row += 1;
                    }
                }
                Key::Char('d') | Key::Char('D') | Key::Right => {
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
