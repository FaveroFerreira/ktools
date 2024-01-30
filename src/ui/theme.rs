use std::path::PathBuf;
use std::str::FromStr;

use anyhow::bail;
use ratatui::style::Modifier;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize)]
#[serde(try_from = "String")]
pub struct Color(ratatui::style::Color);

impl FromStr for Color {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ratatui::style::Color::from_str(s)
            .map(Self)
            .map_err(|_| anyhow::anyhow!("invalid color"))
    }
}

impl TryFrom<String> for Color {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::from_str(&s)
    }
}

impl From<Color> for ratatui::style::Color {
    fn from(value: Color) -> Self {
        value.0
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_string().serialize(serializer)
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(from = "StyleShadow")]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub modifier: Modifier,
}

impl From<Style> for ratatui::style::Style {
    fn from(value: Style) -> Self {
        ratatui::style::Style {
            fg: value.fg.map(Into::into),
            bg: value.bg.map(Into::into),
            underline_color: None,
            add_modifier: value.modifier,
            sub_modifier: Modifier::empty(),
        }
    }
}

#[derive(Default, Deserialize)]
pub(super) struct StyleShadow {
    #[serde(default)]
    pub(super) fg: Option<Color>,
    #[serde(default)]
    pub(super) bg: Option<Color>,
    #[serde(default)]
    pub(super) bold: bool,
    #[serde(default)]
    pub(super) dim: bool,
    #[serde(default)]
    pub(super) italic: bool,
    #[serde(default)]
    pub(super) underline: bool,
    #[serde(default)]
    pub(super) blink: bool,
    #[serde(default)]
    pub(super) blink_rapid: bool,
    #[serde(default)]
    pub(super) reversed: bool,
    #[serde(default)]
    pub(super) hidden: bool,
    #[serde(default)]
    pub(super) crossed: bool,
}

impl From<StyleShadow> for Style {
    fn from(value: StyleShadow) -> Self {
        let mut modifier = Modifier::empty();
        if value.bold {
            modifier |= Modifier::BOLD;
        }
        if value.dim {
            modifier |= Modifier::DIM;
        }
        if value.italic {
            modifier |= Modifier::ITALIC;
        }
        if value.underline {
            modifier |= Modifier::UNDERLINED;
        }
        if value.blink {
            modifier |= Modifier::SLOW_BLINK;
        }
        if value.blink_rapid {
            modifier |= Modifier::RAPID_BLINK;
        }
        if value.reversed {
            modifier |= Modifier::REVERSED;
        }
        if value.hidden {
            modifier |= Modifier::HIDDEN;
        }
        if value.crossed {
            modifier |= Modifier::CROSSED_OUT;
        }

        Self {
            fg: value.fg,
            bg: value.bg,
            modifier,
        }
    }
}

#[derive(Deserialize)]
pub struct Theme {
    // Border
    pub border_style: Style,

    // Tab
    pub tab_active: Style,
    pub tab_inactive: Style,
    pub tab_width: u8,
}

impl Theme {
    pub fn path() -> PathBuf {
        PathBuf::from("./preset/theme.toml")
    }

    pub fn load() -> anyhow::Result<Self> {
        let path = Self::path();

        if !path.exists() {
            bail!("Theme file not found: {:?}", path);
        }

        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
}
