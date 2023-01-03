use termion::color;

pub struct PaletteElement {
    pub fg: color::Fg<&'static dyn color::Color>,
    pub bg: color::Bg<&'static dyn color::Color>,
}

impl PaletteElement {
    pub const fn new(bg: &'static dyn color::Color, fg: &'static dyn color::Color) -> Self {
        Self {
            fg: color::Fg(fg),
            bg: color::Bg(bg),
        }
    }
}

pub struct Palette {
    pub closed: PaletteElement,
    pub open_bg: color::Bg<&'static dyn color::Color>,
    pub neighbour_count_to_fg_color: [color::Fg<&'static dyn color::Color>; 9],
    pub mine: PaletteElement,
    pub flag: PaletteElement,
    pub cursor_fg: color::Fg<&'static dyn color::Color>,
}

pub const OG_PALETTE: Palette = Palette {
    closed: PaletteElement::new(&color::Rgb(30, 30, 30), &color::Rgb(30, 30, 30)),
    open_bg: color::Bg(&color::Rgb(138, 138, 138)),
    neighbour_count_to_fg_color: [
        color::Fg(&color::Reset),               // 0
        color::Fg(&color::Rgb(0, 0, 255)),      // 1
        color::Fg(&color::Rgb(0, 130, 0)),      // 2
        color::Fg(&color::Rgb(200, 0, 0)),      // 3
        color::Fg(&color::Rgb(0, 0, 131)),      // 4
        color::Fg(&color::Rgb(132, 0, 1)),      // 5
        color::Fg(&color::Rgb(0, 130, 132)),    // 6
        color::Fg(&color::Rgb(132, 0, 132)),    // 7
        color::Fg(&color::Rgb(117, 117, 117)),  // 8
    ],
    mine: PaletteElement::new(&color::Red, &color::White),
    flag: PaletteElement::new(&color::Rgb(40, 100, 40), &color::White),
    cursor_fg: color::Fg(&color::White),
};

pub const MNSWPR_PALETTE: Palette = Palette {
    closed: PaletteElement::new(&color::Rgb(30, 30, 30), &color::White),
    open_bg: color::Bg(&color::Rgb(30, 30, 30)),
    neighbour_count_to_fg_color: [
        color::Fg(&color::Reset),               // 0
        color::Fg(&color::Rgb(70, 100, 255)),   // 1
        color::Fg(&color::Rgb(0, 130, 0)),      // 2
        color::Fg(&color::Rgb(200, 0, 0)),      // 3
        color::Fg(&color::Rgb(200, 30, 200)),   // 4
        color::Fg(&color::Rgb(132, 0, 1)),      // 5
        color::Fg(&color::Rgb(0, 130, 132)),    // 6
        color::Fg(&color::Rgb(132, 0, 132)),    // 7
        color::Fg(&color::Rgb(117, 117, 117)),  // 8
    ],
    mine: PaletteElement::new(&color::Red, &color::White),
    flag: PaletteElement::new(&color::Rgb(40, 100, 40), &color::White),
    cursor_fg: color::Fg(&color::White),
};

pub const BG_RESET: color::Bg<color::Reset> = color::Bg(color::Reset);
pub const FG_RESET: color::Fg<color::Reset> = color::Fg(color::Reset);
