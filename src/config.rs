use crate::colors::{Palette, MNSWPR_PALETTE, OG_PALETTE};
use std::path::Path;
use std::str::FromStr;
use std::fmt::Display;
use std::fs;

use anyhow::{ Result, Context };

#[derive(Clone)]
pub enum Theme {
    Mnswpr,
    OG,
    Custom(String),
}

impl Theme {
    pub fn to_palette(&self) -> Result<Palette> {
        match self {
            Theme::Mnswpr => Ok(MNSWPR_PALETTE),
            Theme::OG => Ok(OG_PALETTE),
            Theme::Custom(path) => {
                let theme_data = fs::read_to_string(path).context("Could not read theme data.")?;
                let custom_palette = serde_yaml::from_str(&theme_data)?;
                Ok(custom_palette)
            }
        }
    }
}

impl Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Theme::Mnswpr => write!(f, "mnswpr"),
            Theme::OG => write!(f, "og"),
            Theme::Custom(path) => write!(f, "Custom theme at {path}"),
        }
    }
}

impl FromStr for Theme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mnswpr" => Ok(Self::Mnswpr),
            "og" => Ok(Self::OG),
            path => {
                let path_obj = Path::new(path);
                if !path_obj.exists() {
                    return Err(format!(
                        "Expected one of \"mnswpr\", \"og\", or a custom theme path. Custom theme file at path \"{path}\" not found."
                    ));
                }
                if !path_obj.is_file() {
                    return Err(format!(
                        "The provided custom theme path (\"{path}\") is not a file."
                    ));
                } else {
                    Ok(Self::Custom(path.to_string()))
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum SizePreset {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
}

impl SizePreset {
    /// Returns the pair (columns, rows) corresponding the preset
    pub fn to_size(&self) -> (u64, u64) {
        match self {
            SizePreset::Tiny => (20, 13),
            SizePreset::Small => (30, 20),
            SizePreset::Medium => (40, 25),
            SizePreset::Large => (50, 30),
            SizePreset::Huge => (60, 40),
        }
    }
}

impl Display for SizePreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            SizePreset::Tiny => "tiny",
            SizePreset::Small => "small",
            SizePreset::Medium => "medium",
            SizePreset::Large => "large",
            SizePreset::Huge => "huge",
        };
        write!(f, "{name}")
    }
}

impl FromStr for SizePreset {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "tiny" => Ok(SizePreset::Tiny),
            "small" => Ok(SizePreset::Small),
            "medium" => Ok(SizePreset::Medium),
            "large" => Ok(SizePreset::Large),
            "huge" => Ok(SizePreset::Huge),
            v => Err(format!(
                "Expected one of \"tiny\", \"small\", \"medium\", \"large\", \"huge\". Got \"{v}\""
            )),
        }
    }
}