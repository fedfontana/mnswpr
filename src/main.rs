// #![warn(
//     clippy::all,
//     clippy::restriction,
//     clippy::pedantic,
//     clippy::nursery,
//     clippy::cargo,
// )]

use std::io::{stdin, stdout, Write};

use clap::{command, Parser};

use anyhow::{Context, Result};

use colors::FG_RESET;
use termion::event::{Event, Key};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::{color, cursor::HideCursor};

mod cell;
mod colors;
mod config;
mod field;
mod mnswpr;

use crate::mnswpr::Mnswpr;

use config::{SizePreset, Theme};

/// A simple minesweeper game for the terminal.
///
/// Move the cursor with either wasd, hjkl or the arrows.
///
/// Flag/unflag the cell under the cursor by pressing f, or uncover it by pressing <space> or <insert>.
///
/// Additionally, if you think you have flagged all the mines around a cell, you can press <space> or <enter> on it to open all
/// of the closed cells around it. Note that this will try to open cells that contain mines!
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

    /// The theme of the board
    #[arg(short, long, default_value_t=Theme::Mnswpr)]
    theme: Theme,

    /// If active, trying to flag an open cell with N neighboring mines and N non-open adjacent cells will result in
    /// all of those cells getting flagged
    #[arg(long, default_value_t = false)]
    assisted_flagging: bool,

    /// If active, trying to open an open cell with N neighboring mines and N flagged adjacent cells will result in all
    /// of those cells getting opened
    #[arg(long, default_value_t = false)]
    assisted_opening: bool,
}

/// Returns (cols, rows) after parsing the cli arguments and clipping them with the size of the terminal minus some chars for padding
fn parse_field_size(args: &Args) -> Result<(usize, usize)> {
    let termsize = termion::terminal_size()?;

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

    Ok((cols, rows))
}

fn main() -> Result<()> {
    let args = Args::parse();

    let (cols, rows) = parse_field_size(&args).context("Could not get the size of the terminal")?;

    let mut mnswpr = Mnswpr::new(rows, cols, args.mine_percentage, args.theme.to_palette()?);

    let mut stdout = HideCursor::from(stdout().into_raw_mode()?);

    let mut wins = 0;
    let mut losses = 0;

    loop {
        write!(
            stdout,
            "{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1)
        )?;
        mnswpr.reset();

        let user_did_win = mnswpr.play(&mut stdout, args.assisted_opening, args.assisted_flagging)?;

        // If the user explicitly quit, then exit out of the program
        if user_did_win.is_none() {
            break;
        }

        mnswpr.print_game_state(&mut stdout, true)?;
        if user_did_win.unwrap() {
            write!(stdout, "{}You won!{FG_RESET}\r\n", color::Fg(color::Green))?;
            wins += 1;
        } else {
            write!(
                stdout,
                "{}You lost!{FG_RESET}\r\n",
                color::Fg(color::LightRed),
            )?;
            losses += 1;
        }
        write!(
            stdout,
            "Wins: {wins}    Losses: {losses}   Win rate: {:.1}%\r\n",
            wins as f64 / (wins + losses) as f64 * 100_f64
        )?;
        write!(
            stdout,
            "Press y/Y/<space>/<insert> if you want to play again, otherwise press n/N\r\n"
        )?;
        stdout.flush()?;

        let stdin = stdin();
        for e in stdin.events() {
            if let Event::Key(event) = e? {
                match event {
                    Key::Char(' ' | 'y' | 'Y' | '\n') => {
                        break;
                    }
                    Key::Char('q' | 'Q' | 'n' | 'N') => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
