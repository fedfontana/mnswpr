use termion::color;
use serde::{Deserialize, Deserializer};
use std::fmt;
use serde::de::{self, Visitor};

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

impl<'de> Deserialize<'de> for color::Rgb {
    fn deserialize<D>(deserializer: D) -> Result<i32, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(HexColorVisitor)
    }
}

struct HexColorVisitor;
impl<'de> Visitor<'de> for HexColorVisitor {
    type Value = color::Rgb;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a color in the format #rrggbb")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error, {
                let s = v.to_lowercase();
                if s.len() != 7 {
                    return Err(E::custom("hex color must have lenght 7"));
                }
                
                let r = u8::from_str_radix(&s[1..3], 16);
                let g = u8::from_str_radix(&s[3..5], 16);
                let b = u8::from_str_radix(&s[5..7], 16);
                
                if r.is_err() || g.is_err() || b.is_err() {
                    return Err(E::custom("color values out of range"));
                }
            
                Ok(color::Rgb(r.unwrap(), g.unwrap(), b.unwrap()))
    }
}

pub struct Palette {
    pub closed: PaletteElement,
    pub open_bg: color::Bg<&'static dyn color::Color>,
    pub neighbour_count_to_fg_color: [color::Fg<&'static dyn color::Color>; 9],
    pub mine: PaletteElement,
    pub flag: PaletteElement,
    pub cursor_fg: color::Fg<&'static dyn color::Color>,
    pub correct_flag: PaletteElement,
    pub wrong_flag: PaletteElement,
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
    correct_flag: PaletteElement::new(&color::Green, &color::White),
    wrong_flag: PaletteElement::new(&color::LightRed, &color::White),
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
    correct_flag: PaletteElement::new(&color::Green, &color::White),
    wrong_flag: PaletteElement::new(&color::LightRed, &color::White),
};

pub const BG_RESET: color::Bg<color::Reset> = color::Bg(color::Reset);
pub const FG_RESET: color::Fg<color::Reset> = color::Fg(color::Reset);
