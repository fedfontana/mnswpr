use crate::colors::{self, BG_RESET, FG_RESET};
use termion::color;

#[derive(Copy, Clone, Debug)]
pub enum State {
    Open,
    Closed,
    Flagged,
}

impl Default for State {
    fn default() -> Self {
        Self::Closed
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Content {
    Mine,
    Empty,
}

impl Default for Content {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Cell {
    pub state: State,
    pub content: Content,
    pub neighbouring_bomb_count: usize,
}

// impl Display for Cell {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self.state {
//             CellState::Open => match self.content {
//                 CellContent::Mine => {
//                     write!(f, "{}*{}", color::Bg(color::Red), color::Bg(color::Reset))
//                 }
//                 CellContent::Empty => write!(
//                     f,
//                     "{}{}{}{}{}",
//                     BG_COLOR,
//                     NBOR_COUNT_TO_FG_COLOR[self.neighbouring_bomb_count],
//                     if self.neighbouring_bomb_count != 0 { self.neighbouring_bomb_count.to_string() } else { " ".to_string() },
//                     color::Fg(color::Reset),
//                     color::Bg(color::Reset)
//                 ),
//             },
//             CellState::Closed => write!(f, "."),
//             CellState::Flagged => {
//                 write!(f, "{}F{}", color::Bg(color::Blue), color::Bg(color::Reset))
//             }
//         }
//     }
// }

impl Cell {
    pub fn set_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    pub fn to_string_with_palette(&self, palette: &colors::Palette, with_cursor: bool) -> String {
        let sep = if with_cursor {
            ('[', ']')
        } else {
            (' ', ' ')
        };

        match self.state {
            State::Open => match self.content {
                Content::Mine => {
                    format!("{}{}*{}{}", palette.mine_bg, sep.0, sep.1, BG_RESET)
                }
                Content::Empty => format!(
                    "{bg}{sep0}{fg}{repr}{FG_RESET}{sep1}{BG_RESET}",
                    bg = palette.bg,
                    sep0 = sep.0,
                    fg = palette.neighbour_count_to_fg_color[self.neighbouring_bomb_count],
                    sep1 = sep.1,
                    repr = if self.neighbouring_bomb_count != 0 {
                        self.neighbouring_bomb_count.to_string()
                    } else {
                        " ".to_string()
                    }
                ),
            },
            State::Closed => format!("{}{}.{}{}", palette.bg, sep.0, sep.1, BG_RESET),
            State::Flagged => {
                format!("{}{}F{}{BG_RESET}", palette.flag_bg, sep.0, sep.1)
            }
        }
    }

    pub fn to_string_with_palette_lost(&self, palette: &colors::Palette, with_cursor: bool) -> String {
        let sep = if with_cursor {
            ('[', ']')
        } else {
            (' ', ' ')
        };

        match (self.state, self.content) {
            (State::Flagged, Content::Mine) => format!("{}{}*{}{BG_RESET}", sep.0, sep.1, color::Bg(color::Green)),
            (State::Flagged, Content::Empty) => {
                format!(
                    "{bg}{}{count}{}{BG_RESET}",
                    sep.0,
                    sep.1,
                    bg=color::Bg(color::LightRed),
                    count=self.neighbouring_bomb_count,
                )
            }
            (_, Content::Mine) => format!("{}{}*{}{BG_RESET}", palette.mine_bg, sep.0, sep.1),
            (_, Content::Empty) => format!(
                "{bg}{}{fg}{count}{FG_RESET}{}{BG_RESET}",
                sep.0,
                sep.1,
                bg = palette.bg,
                fg = palette.neighbour_count_to_fg_color[self.neighbouring_bomb_count],
                count = if self.neighbouring_bomb_count != 0 {
                    self.neighbouring_bomb_count.to_string()
                } else {
                    " ".to_string()
                },
            ),
        }
    }
}
