///! Handle the color theme
use crate::skim::options::SkimOptions;
use tuikit::prelude::*;

#[rustfmt::skip]
lazy_static! {
    pub static ref DEFAULT_THEME:  ColorTheme = ColorTheme::default();
}

/// The color scheme of skim's UI
///
/// <pre>
/// +----------------+
/// | >selected line |  --> selected & normal(fg/bg) & matched
/// |> current line  |  --> cursor & current & current_match
/// |  normal line   |
/// |\ 8/10          |  --> spinner & info
/// |> query         |  --> prompt & query
/// +----------------+
/// </pre>
#[rustfmt::skip]
#[derive(Copy, Clone, Debug)]
pub struct ColorTheme {
    fg:                   Color,
    bg:                   Color,
    normal_effect:        Effect,
    matched:              Color,
    matched_bg:           Color,
    matched_effect:       Effect,
    current:              Color,
    current_bg:           Color,
    current_effect:       Effect,
    current_match:        Color,
    current_match_bg:     Color,
    current_match_effect: Effect,
    query_fg:             Color,
    query_bg:             Color,
    query_effect:         Effect,
    spinner:              Color,
    info:                 Color,
    prompt:               Color,
    cursor:               Color,
    selected:             Color,
    header:               Color,
    border:               Color,
}

#[rustfmt::skip]
#[allow(dead_code)]
impl ColorTheme {
    pub fn init_from_options(options: &SkimOptions) -> ColorTheme {
        // register
        if let Some(color) = options.color {
            ColorTheme::from_options(color)
        } else {
            ColorTheme::default()
        }
    }
    
    fn empty() -> Self {
        ColorTheme {
            fg:                   Color::Default,
            bg:                   Color::Default,
            normal_effect:        Effect::empty(),
            matched:              Color::Default,
            matched_bg:           Color::Default,
            matched_effect:       Effect::empty(),
            current:              Color::Default,
            current_bg:           Color::Default,
            current_effect:       Effect::empty(),
            current_match:        Color::Default,
            current_match_bg:     Color::Default,
            current_match_effect: Effect::empty(),
            query_fg:             Color::Default,
            query_bg:             Color::Default,
            query_effect:         Effect::empty(),
            spinner:              Color::Default,
            info:                 Color::Default,
            prompt:               Color::Default,
            cursor:               Color::Default,
            selected:             Color::Default,
            header:               Color::Default,
            border:               Color::Default,
        }
    }

    #[allow(clippy::wildcard_in_or_patterns)]
    fn from_options(color: &str) -> Self {
        let mut theme = ColorTheme::default();
        for pair in color.split(',') {
            let color: Vec<&str> = pair.split(':').collect();

            let new_color = if color[1].len() == 7 {
                // 256 color
                let r = u8::from_str_radix(&color[1][1..3], 16).unwrap_or(255);
                let g = u8::from_str_radix(&color[1][3..5], 16).unwrap_or(255);
                let b = u8::from_str_radix(&color[1][5..7], 16).unwrap_or(255);
                Color::Rgb(r, g, b)
            } else {
                color[1].parse::<u8>()
                    .map(Color::AnsiValue)
                    .unwrap_or(Color::Default)
            };

            match color[0] {
                "fg"                    => theme.fg               = new_color,
                "bg"                    => theme.bg               = new_color,
                "matched" | "hl"        => theme.matched          = new_color,
                "matched_bg"            => theme.matched_bg       = new_color,
                "current" | "fg+"       => theme.current          = new_color,
                "current_bg" | "bg+"    => theme.current_bg       = new_color,
                "current_match" | "hl+" => theme.current_match    = new_color,
                "current_match_bg"      => theme.current_match_bg = new_color,
                "query"                 => theme.query_fg         = new_color,
                "query_bg"              => theme.query_bg         = new_color,
                "spinner"               => theme.spinner          = new_color,
                "info"                  => theme.info             = new_color,
                "prompt"                => theme.prompt           = new_color,
                "cursor" | "pointer"    => theme.cursor           = new_color,
                "selected" | "marker"   => theme.selected         = new_color,
                "header"                => theme.header           = new_color,
                "border"                => theme.border           = new_color,
                _ => {}
            }
        }
        theme
    }

    pub fn normal(&self) -> Attr {
        Attr {
            fg: self.fg,
            bg: self.bg,
            effect: self.normal_effect,
        }
    }

    pub fn matched(&self) -> Attr {
        Attr {
            fg: self.matched,
            bg: self.matched_bg,
            effect: self.matched_effect,
        }
    }

    pub fn current(&self) -> Attr {
        Attr {
            fg: self.current,
            bg: self.current_bg,
            effect: self.current_effect,
        }
    }

    pub fn current_match(&self) -> Attr {
        Attr {
            fg: self.current_match,
            bg: self.current_match_bg,
            effect: self.current_match_effect,
        }
    }

    pub fn query(&self) -> Attr {
        Attr {
            fg: self.query_fg,
            bg: self.query_bg,
            effect: self.query_effect,
        }
    }

    pub fn spinner(&self) -> Attr {
        Attr {
            fg: self.spinner,
            bg: self.bg,
            effect: Effect::BOLD,
        }
    }

    pub fn info(&self) -> Attr {
        Attr {
            fg: self.info,
            bg: self.bg,
            effect: Effect::empty(),
        }
    }

    pub fn prompt(&self) -> Attr {
        Attr {
            fg: self.prompt,
            bg: self.bg,
            effect: Effect::empty(),
        }
    }

    pub fn cursor(&self) -> Attr {
        Attr {
            fg: self.cursor,
            bg: self.current_bg,
            effect: Effect::empty(),
        }
    }

    pub fn selected(&self) -> Attr {
        Attr {
            fg: self.selected,
            bg: self.current_bg,
            effect: Effect::empty(),
        }
    }

    pub fn header(&self) -> Attr {
        Attr {
            fg: self.header,
            bg: self.bg,
            effect: Effect::empty(),
        }
    }

    pub fn border(&self) -> Attr {
        Attr {
            fg: self.border,
            bg: self.bg,
            effect: Effect::empty(),
        }
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        ColorTheme {
            normal_effect: Effect::DIM,
            matched: Color::CYAN,
            current_effect: Effect::BOLD,
            current_match: Color::CYAN,
            info: Color::YELLOW,

            cursor: Color::GREEN,
            ..ColorTheme::empty()
        }
    }
}
