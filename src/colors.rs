use termion::color;
use serde::{Deserialize, Deserializer};
use std::fmt;
use serde::de::{self, Visitor};

#[derive(Deserialize, Debug)]
pub struct PaletteElement {
    pub fg: CFg,
    pub bg: CBg,
}

impl PaletteElement {
    pub const fn new(bg: color::Rgb, fg: color::Rgb) -> Self {
        Self {
            fg: CFg(color::Fg(fg)),
            bg: CBg(color::Bg(bg)),
        }
    }
}

#[derive(Debug)]
pub struct CFg(pub color::Fg<color::Rgb>);

impl CFg {
    pub const fn new(clr: color::Rgb) -> Self {
        Self(color::Fg(clr))
    }
}

#[derive(Debug)]
pub struct CBg(pub color::Bg<color::Rgb>);

impl CBg {
    pub const fn new(clr: color::Rgb) -> Self {
        Self(color::Bg(clr))
    }
}

impl<'de> Deserialize<'de> for CFg {
    fn deserialize<D>(deserializer: D) -> Result<CFg, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(CFg::new(deserializer.deserialize_str(HexColorVisitor)?))
    }
}

impl<'de> Deserialize<'de> for CBg {
    fn deserialize<D>(deserializer: D) -> Result<CBg, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(CBg::new(deserializer.deserialize_str(HexColorVisitor)?))
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

#[derive(Deserialize, Debug)]
pub struct Palette {
    pub closed: PaletteElement,
    pub open_bg: CBg,
    pub neighbour_count_to_fg_color: [CFg; 9],
    pub mine: PaletteElement,
    pub flag: PaletteElement,
    pub cursor_fg: CFg,
    pub correct_flag: PaletteElement,
    pub wrong_flag: PaletteElement,
}

pub const OG_PALETTE: Palette = Palette {
    closed: PaletteElement::new(color::Rgb(30, 30, 30), color::Rgb(30, 30, 30)),
    open_bg: CBg::new(color::Rgb(138, 138, 138)),
    neighbour_count_to_fg_color: [
        CFg::new(color::Rgb(138, 138, 138)),               // 0
        CFg::new(color::Rgb(0, 0, 255)),      // 1
        CFg::new(color::Rgb(0, 130, 0)),      // 2
        CFg::new(color::Rgb(200, 0, 0)),      // 3
        CFg::new(color::Rgb(0, 0, 131)),      // 4
        CFg::new(color::Rgb(132, 0, 1)),      // 5
        CFg::new(color::Rgb(0, 130, 132)),    // 6
        CFg::new(color::Rgb(132, 0, 132)),    // 7
        CFg::new(color::Rgb(117, 117, 117)),  // 8
    ],
    mine: PaletteElement::new(color::Rgb(180, 0, 0), color::Rgb(255, 255, 255)),
    flag: PaletteElement::new(color::Rgb(40, 100, 40), color::Rgb(255, 255, 255)),
    cursor_fg: CFg::new(color::Rgb(255, 255, 255)),
    correct_flag: PaletteElement::new(color::Rgb(0, 255, 0), color::Rgb(255, 255, 255)),
    wrong_flag: PaletteElement::new(color::Rgb(255, 0, 0), color::Rgb(255, 255, 255)),
};

pub const MNSWPR_PALETTE: Palette = Palette {
    closed: PaletteElement::new(color::Rgb(30, 30, 30), color::Rgb(255, 255, 255)),
    open_bg: CBg::new(color::Rgb(30, 30, 30)),
    neighbour_count_to_fg_color: [
        CFg::new(color::Rgb(30, 30, 30)),     // 0
        CFg::new(color::Rgb(70, 100, 255)),   // 1
        CFg::new(color::Rgb(0, 130, 0)),      // 2
        CFg::new(color::Rgb(200, 0, 0)),      // 3
        CFg::new(color::Rgb(200, 30, 200)),   // 4
        CFg::new(color::Rgb(132, 0, 1)),      // 5
        CFg::new(color::Rgb(0, 130, 132)),    // 6
        CFg::new(color::Rgb(132, 0, 132)),    // 7
        CFg::new(color::Rgb(117, 117, 117)),  // 8
    ],
    mine: PaletteElement::new(color::Rgb(180, 0, 0), color::Rgb(255, 255, 255)),
    flag: PaletteElement::new(color::Rgb(40, 100, 40), color::Rgb(255, 255, 255)),
    cursor_fg: CFg::new(color::Rgb(255, 255, 255)),
    correct_flag: PaletteElement::new(color::Rgb(0, 255, 0), color::Rgb(255, 255, 255)),
    wrong_flag: PaletteElement::new(color::Rgb(255, 0, 0), color::Rgb(255, 255, 255)),
};

pub const BG_RESET: color::Bg<color::Reset> = color::Bg(color::Reset);
pub const FG_RESET: color::Fg<color::Reset> = color::Fg(color::Reset);
