use crate::colors;

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

impl Cell {
    pub fn set_state(&mut self, new_state: State) {
        self.state = new_state;
    }

    /// This method does not reset the fg/bg color!!
    pub fn to_string_with_palette(&self, palette: &colors::Palette, with_cursor: bool) -> String {
        let sep = if with_cursor { ('[', ']') } else { (' ', ' ') };
        let cursor = (
            format!("{}{}", palette.cursor_fg.0, sep.0),
            format!("{}{}", palette.cursor_fg.0, sep.1),
        );

        let bg;
        let fg;
        let repr;

        match self.state {
            State::Open => match self.content {
                Content::Mine => {
                    bg = &palette.mine.bg;
                    fg = &palette.mine.fg;
                    repr = "*".to_string();
                }
                Content::Empty => {
                    bg = &palette.open_bg;
                    fg = &palette.neighbour_count_to_fg_color[self.neighbouring_bomb_count];
                    repr = if self.neighbouring_bomb_count != 0 {
                        self.neighbouring_bomb_count.to_string()
                    } else {
                        " ".to_string()
                    };
                }
            },
            State::Closed => {
                bg = &palette.closed.bg; 
                fg = &palette.closed.fg; 
                repr = ".".to_string();
            }
            State::Flagged => {
                bg = &palette.flag.bg;
                fg = &palette.flag.fg;
                repr = "F".to_string();
            }
        };
        format!(
            "{bg}{cursor0}{fg}{repr}{cursor1}",
            bg = bg.0,
            fg = fg.0,
            cursor0 = cursor.0,
            cursor1 = cursor.1,
        )
    }

    pub fn to_string_with_palette_lost(
        &self,
        palette: &colors::Palette,
        with_cursor: bool,
    ) -> String {
        let sep = if with_cursor { ('[', ']') } else { (' ', ' ') };
        let cursor = (
            format!("{}{}", palette.cursor_fg.0, sep.0),
            format!("{}{}", palette.cursor_fg.0, sep.1),
        );

        let bg;
        let fg;
        let repr;

        match (self.state, self.content) {
            (State::Flagged, Content::Mine) => {
                bg = &palette.correct_flag.bg;
                fg = &palette.correct_flag.fg;
                repr = "*".to_string();
            }
            (State::Flagged, Content::Empty) => {
                bg = &palette.wrong_flag.bg;
                fg = &palette.wrong_flag.fg;
                repr = self.neighbouring_bomb_count.to_string();
            }
            (_, Content::Mine) => {
                bg = &palette.mine.bg;
                fg = &palette.mine.fg;
                repr = "*".to_string();
            }
            (_, Content::Empty) => {
                bg = &palette.open_bg;
                fg = &palette.neighbour_count_to_fg_color[self.neighbouring_bomb_count];
                repr = if self.neighbouring_bomb_count != 0 {
                    self.neighbouring_bomb_count.to_string()
                } else {
                    " ".to_string()
                };
            }
        };

        format!(
            "{bg}{cursor0}{fg}{repr}{cursor1}",
            bg = bg.0,
            fg = fg.0,
            cursor0 = cursor.0,
            cursor1 = cursor.1,
        )
    }
}
