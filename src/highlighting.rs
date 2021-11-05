use crossterm::style::Color;
use crossterm::style::Color::Rgb;

#[derive(PartialEq, Clone, Copy)]
pub enum Type {
    None,
    Number,
    Match,
    String,
    Character,
    Comment,
    MultilineComment,
    PrimaryKeywords,
    SecondaryKeywords,
}

impl Type {
    pub fn to_color(self) -> Color {
        use Type::*;
        match self {
            Number => Rgb {
                r: 220,
                g: 163,
                b: 163,
            },
            Match => Rgb {
                r: 38,
                g: 139,
                b: 210,
            },
            String => Rgb {
                r: 211,
                g: 54,
                b: 130,
            },
            Character => Rgb {
                r: 108,
                g: 113,
                b: 196,
            },
            Comment | MultilineComment => Rgb {
                r: 133,
                g: 153,
                b: 0,
            },
            PrimaryKeywords => Rgb {
                r: 181,
                g: 137,
                b: 0,
            },
            SecondaryKeywords => Rgb {
                r: 42,
                g: 161,
                b: 152,
            },
            _ => Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
        }
    }
}
