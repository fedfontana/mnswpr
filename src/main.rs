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

use field::{ Field, CellContent, CellState, Cell };



//TODO split into files
//TODO Game struct should be different from Field struct
//TODO refactor

//TODO add mouse click support (supported by termion)?
//TODO add docs and tests
//TODO add cli options:
//  - widht
//  - height
//  - bomb_percentage
//  - presets like easy, hard, medium, ...
//  - auto-counter or something like that. When this option is active, numbers that have too many flags around them turn bright red,
//          and the ones with the right amount of flags around them get printed green

//TODO add a retry option after concluding a match
//TODO when a match is lost, highlight the flags in the wrong place with a different color
//TODO add bg color for uncovered cells (the original gray (185, 185, 185))
//TODO generate board when clicking on first cell. 
//  - Either generate a number or a whole area of numbers under the cursor in a way that the first tile cannot be a bomb.



fn main() {
    let mut field = Field::new(10, 10, 20);

    let stdin = stdin();
    let mut stdout = HideCursor::from(stdout().into_raw_mode().unwrap());

    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Goto(1,1));

    write!(
        stdout,
        "{}Mines:{}    Flags:{}\r\n{field}",
        termion::cursor::Goto(1, 1),
        field.mine_count,
        field.flag_count
    );
    stdout.flush().unwrap();

    for c in stdin.events() {
        if let Event::Key(event) = c.unwrap() {
            match event {
                Key::Char('q') | Key::Char('Q') => break,
                Key::Char('w') | Key::Char('W') | Key::Up => {
                    if field.cursor.row > 0 {
                        field.cursor.row -= 1;
                    }
                }
                Key::Char('a') | Key::Char('A') | Key::Left => {
                    if field.cursor.col > 0 {
                        field.cursor.col -= 1;
                    }
                }
                Key::Char('s') | Key::Char('S') | Key::Down => {
                    if field.cursor.row < field.rows - 1 {
                        field.cursor.row += 1;
                    }
                }
                Key::Char('d') | Key::Char('D') | Key::Right => {
                    if field.cursor.col < field.cols - 1 {
                        field.cursor.col += 1;
                    }
                }
                Key::Char(' ') => match field.uncover_at_cursor() {
                    CellContent::Mine => {
                        let cell = field.get(field.cursor.row, field.cursor.col).unwrap();
                        if !matches!(cell.state, CellState::Flagged) {
                            field.uncover_all();
                            write!(
                                stdout,
                                "{}{}Mines:{}    Flags:{}\r\n{field}",
                                termion::cursor::Goto(1, 1),
                                termion::clear::CurrentLine,
                                field.mine_count,
                                field.flag_count
                            );
                            write!(
                                stdout,
                                "{}You lost!{}\r\n",
                                color::Fg(color::LightRed),
                                color::Fg(color::Reset)
                            );
                            break;
                        }
                    }
                    CellContent::Empty => {}
                },
                Key::Char('f') | Key::Char('F') => field.toggle_flag_at_cursor(),
                _ => {}
            }
        }
        write!(
            stdout,
            "{}{}Mines:{}    Flags:{}\r\n{field}",
            termion::cursor::Goto(1, 1),
            termion::clear::CurrentLine,
            field.mine_count,
            field.flag_count
        );
        if field.covered_empty_cells == 0 {
            write!(
                stdout,
                "{}You won!{}\r\n",
                color::Fg(color::Green),
                color::Fg(color::Reset)
            );
            stdout.flush().unwrap();
            break;
        }
        stdout.flush().unwrap();
    }
}
