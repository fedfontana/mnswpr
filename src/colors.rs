use termion::color;

pub struct Palette {
    pub bg: color::Bg<&'static dyn color::Color>,
    pub neighbour_count_to_fg_color: [color::Fg<&'static dyn color::Color>; 9],
    pub mine_bg: color::Bg<&'static dyn color::Color>,
    pub flag_bg: color::Bg<&'static dyn color::Color>,
}

pub const DEFAULT_PALETTE: Palette = Palette {
    bg: color::Bg(&color::Rgb(30, 30, 30)),
    neighbour_count_to_fg_color: [
        color::Fg(&color::Reset),              // 0
        color::Fg(&color::Rgb(70, 70, 255)),   // 1 // original is Rgb(0, 0, 255)
        color::Fg(&color::Rgb(0, 130, 0)),     // 2
        color::Fg(&color::Rgb(200, 0, 0)),     // 3
        color::Fg(&color::Rgb(50, 50, 131)),   // 4 // original is Rgb(0, 0, 131)
        color::Fg(&color::Rgb(132, 0, 1)),     // 5
        color::Fg(&color::Rgb(0, 130, 132)),   // 6
        color::Fg(&color::Rgb(132, 0, 132)),   // 7
        color::Fg(&color::Rgb(117, 117, 117)), // 8
    ],
    mine_bg: color::Bg(&color::Red),
    flag_bg: color::Bg(&color::Blue),
};

pub const BG_RESET: color::Bg<color::Reset> = color::Bg(color::Reset);
pub const FG_RESET: color::Fg<color::Reset> = color::Fg(color::Reset);
