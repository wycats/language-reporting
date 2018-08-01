use std::fmt;
use std::str::FromStr;
use termcolor;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Color {
    Black,
    Blue,
    Green,
    Red,
    Cyan,
    Magenta,
    Yellow,
    White,
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let out = match self {
            Color::Black => "black",
            Color::Blue => "blue",
            Color::Green => "green",
            Color::Red => "red",
            Color::Cyan => "cyan",
            Color::Magenta => "magenta",
            Color::Yellow => "yellow",
            Color::White => "white",
        };

        write!(f, "{}", out)
    }
}

impl From<Color> for termcolor::Color {
    fn from(color: Color) -> termcolor::Color {
        match color {
            Color::Black => termcolor::Color::Black,
            Color::Blue => if cfg!(windows) {
                termcolor::Color::Cyan
            } else {
                termcolor::Color::Blue
            },
            Color::Green => termcolor::Color::Green,
            Color::Red => termcolor::Color::Red,
            Color::Cyan => termcolor::Color::Cyan,
            Color::Magenta => termcolor::Color::Magenta,
            Color::Yellow => termcolor::Color::Yellow,
            Color::White => termcolor::Color::White,
        }
    }
}

impl FromStr for Color {
    type Err = (&'static str, String);

    fn from_str(s: &str) -> Result<Color, (&'static str, String)> {
        match &*s.to_lowercase() {
            "black" => Ok(Color::Black),
            "blue" => Ok(Color::Blue),
            "green" => Ok(Color::Green),
            "red" => Ok(Color::Red),
            "cyan" => Ok(Color::Cyan),
            "magenta" => Ok(Color::Magenta),
            "yellow" => Ok(Color::Yellow),
            "white" => Ok(Color::White),
            _ => Err(("invalid color", s.to_string())),
        }
    }
}

impl<'a> From<&'a str> for Color {
    fn from(s: &str) -> Color {
        Color::from_str(s).unwrap()
    }
}

impl<'a> From<&'a termcolor::Color> for Color {
    fn from(color: &'a termcolor::Color) -> Color {
        match color {
            termcolor::Color::Black => Color::Black,
            termcolor::Color::Blue => Color::Blue,
            termcolor::Color::Green => Color::Green,
            termcolor::Color::Red => Color::Red,
            termcolor::Color::Cyan => Color::Cyan,
            termcolor::Color::Magenta => Color::Magenta,
            termcolor::Color::Yellow => Color::Yellow,
            termcolor::Color::White => Color::White,

            other => panic!(
                "termcolor {:?} is a non-portable color and cannot be converted",
                other
            ),
        }
    }
}
