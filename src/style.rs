use crate::{FromBinary, ToBinary};
/// A type for changing how the text printed by the terminal looks.
///
/// You can change the foreground color, whether or not the foreground uses an intense color,
/// whether or not the foreground uses a dim color, the background color, whether or not the
/// background uses an intense color, whether or not the text is bold, whether or not the text is
/// italic, and whether or not the text is underlined.
///
/// Once you have set those to be how you want them, print your [Style] instance before the text
/// and the visuals will have been changed.
/// ```
/// # use abes_nice_things::Style;
/// # fn main() {
/// println!("{}I'm red!{}", *Style::new().red(), Style::new());
/// # }
/// ```
/// Although, me personally I just put the reset code in directly because it means my [println] is
/// less cluttered.
/// ```
/// # use abes_nice_things::Style;
/// # fn main() {
/// println!("{}I'm also red!\x1b[0m", *Style::new().red());
/// # }
/// ```
/// But that does have the downside of maybe being less readable so it is up to you.
///
/// Bit of a warning: Once you change the style, it will not change back to default by itself, you
/// have to change it. So don't forget to reset the styling with either of the two ways above.
/// Especially be careful because if your code closes before it resets the styling, then the user
/// might be confused and not know why their terminal is acting weird.
///
/// Also, unless otherwise specified, all methods for this act on the foreground (you know, the
/// text)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    color: Color,
    intense: bool,
    background: Color,
    intense_background: bool,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
}
macro_rules! color {
    ($background: ident, $name: ident, $variant: ident) => {
        pub const fn $name(&mut self) -> &mut Self {
            self.color = Color::$variant;
            self
        }
        pub const fn $background(&mut self) -> &mut Self {
            self.background = Color::$variant;
            self
        }
    };
}
macro_rules! set {
    ($name: ident) => {
        pub const fn $name(&mut self, $name: bool) -> &mut Self {
            self.$name = $name;
            self
        }
    };
}
impl Style {
    /// Creates a new [Style] instance which if used will reset the styling.
    ///
    /// Although if you are going to use it to reset the styling then I personally like just
    /// printing "\x1b[0m" because it does the same thing.
    pub const fn new() -> Style {
        Style {
            color: Color::Default,
            intense: false,
            background: Color::Default,
            intense_background: false,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
        }
    }
    /// Sets the foreground color to the given [Color].
    ///
    /// This is helpful for more modular code, but if you are going to just set it to a hard coded
    /// color then I would recommend just using the dedicated function for that color like
    /// [Style::red].
    pub const fn set_color(&mut self, color: Color) -> &mut Self {
        self.color = color;
        self
    }
    /// Sets the background color to the given [Color].
    ///
    /// If you are just going to give this a hard coded color then I would recommend using one of
    /// the dedicated functions like [Style::background_red]
    pub const fn set_background(&mut self, color: Color) -> &mut Self {
        self.background = color;
        self
    }
    /// Returns true if the background has been set.
    pub const fn has_background(&self) -> bool {
        if let Color::Default = self.background {
            return false;
        }
        true
    }
    /// Swaps the background color and foreground color, as well as whether or not they are
    /// intense. This does not swap whether or not they are dim though because you cannot have a
    /// dim background.
    pub const fn swap_grounds(&mut self) -> &mut Self {
        std::mem::swap(&mut self.color, &mut self.background);
        std::mem::swap(&mut self.intense, &mut self.intense_background);
        self
    }

    set!(intense);
    set!(intense_background);
    set!(bold);
    set!(dim);
    set!(italic);
    set!(underline);

    color!(background_black, black, Black);
    color!(background_red, red, Red);
    color!(background_green, green, Green);
    color!(background_yellow, yellow, Yellow);
    color!(background_blue, blue, Blue);
    color!(background_purple, purple, Purple);
    color!(background_cyan, cyan, Cyan);
    color!(background_white, white, White);
    color!(background_default, default, Default);
}
impl std::fmt::Display for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut color = 0;
        if let Some(num) = self.color.to_num() {
            color = num;
            if self.intense {
                color += 60;
            }
        }
        let mut first = true;
        write!(f, "\x1b[")?;
        if self.bold {
            if !first {
                write!(f, ";")?;
            }
            write!(f, "1")?;
            first = false;
        }
        if self.dim {
            if !first {
                write!(f, ";")?;
            }
            write!(f, "2")?;
            first = false;
        }
        if self.italic {
            if !first {
                write!(f, ";")?;
            }
            write!(f, "3")?;
            first = false;
        }
        if self.underline {
            if !first {
                write!(f, ";")?;
            }
            write!(f, "4")?;
            first = false;
        }
        match self.background.to_num() {
            Some(mut background) => {
                if self.intense_background {
                    background += 60
                }
                background += 10;
                if !first {
                    write!(f, ";")?;
                }
                if color != 0 {
                    write!(f, "{color};{background}")?;
                } else {
                    write!(f, "{background}")?;
                }
            }
            None => {
                if color != 0 {
                    if !first {
                        write!(f, ";")?;
                    }
                    write!(f, "{color}")?;
                }
            }
        }
        write!(f, "m")
    }
}
impl FromBinary for Style {
    fn from_binary(binary: &mut dyn std::io::Read) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(Style {
            color: Color::from_binary(binary)?,
            intense: bool::from_binary(binary)?,
            background: Color::from_binary(binary)?,
            intense_background: bool::from_binary(binary)?,
            bold: bool::from_binary(binary)?,
            dim: bool::from_binary(binary)?,
            italic: bool::from_binary(binary)?,
            underline: bool::from_binary(binary)?,
        })
    }
}
impl ToBinary for Style {
    fn to_binary(&self, binary: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        self.color.to_binary(binary)?;
        self.intense.to_binary(binary)?;
        self.background.to_binary(binary)?;
        self.intense_background.to_binary(binary)?;
        self.bold.to_binary(binary)?;
        self.dim.to_binary(binary)?;
        self.italic.to_binary(binary)?;
        self.underline.to_binary(binary)
    }
}
/// The colors used for [Style].
///
/// Default is special because it will be whatever the default state for your terminal is. If you
/// have a dark mode terminal, then it will probably be white, and if you have a light mode
/// terminal, it will probably be black. All you really need to know is that it will be readable
/// against the default background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Purple,
    Cyan,
    White,
    Default,
}
impl Color {
    fn to_num(self) -> Option<u8> {
        match self {
            Color::Black => Some(30),
            Color::Red => Some(31),
            Color::Green => Some(32),
            Color::Yellow => Some(33),
            Color::Blue => Some(34),
            Color::Purple => Some(35),
            Color::Cyan => Some(36),
            Color::White => Some(37),
            Color::Default => None,
        }
    }
}
impl FromBinary for Color {
    fn from_binary(binary: &mut dyn std::io::Read) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(match u8::from_binary(binary)? {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Purple,
            6 => Color::Cyan,
            7 => Color::White,
            8 => Color::Default,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Could not get Color from binary",
                ));
            }
        })
    }
}
impl ToBinary for Color {
    fn to_binary(&self, binary: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        match self {
            Color::Black => 0_u8,
            Color::Red => 1_u8,
            Color::Green => 2_u8,
            Color::Yellow => 3_u8,
            Color::Blue => 4_u8,
            Color::Purple => 5_u8,
            Color::Cyan => 6_u8,
            Color::White => 7_u8,
            Color::Default => 8_u8,
        }
        .to_binary(binary)
    }
}
