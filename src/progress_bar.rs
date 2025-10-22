use crate::numbers::*;
use crate::setter;
use crate::Style;
use crate::{FromBinary, ToBinary};
use std::io::Write;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProgressBar<T: UnsignedInteger> {
    current: T,
    target: T,
    visual_len: T,
    percent_done: bool,
    amount_done: bool,
    done_style: Style,
    /// [Style] used by the characters in the bar that are not done yet
    waiting_style: Style,
    base_style: Style,
    supplementary_newline: bool,
    done_char: char,
    /// Last done character in the bar
    header_char: char,
    waiting_char: char,
    rate: Option<Rate>,
    eta: bool,
    prev: Option<(std::time::SystemTime, T)>,
}
impl<T: UnsignedInteger> ProgressBar<T> {
    pub fn new(target: T, visual_len: T) -> ProgressBar<T> {
        ProgressBar {
            current: T::prim_from(0),
            target,
            visual_len,
            percent_done: false,
            amount_done: false,
            done_style: Style::new(),
            waiting_style: Style::new(),
            base_style: Style::new(),
            supplementary_newline: false,
            done_char: '=',
            header_char: '>',
            waiting_char: '-',
            rate: None,
            eta: false,
            prev: None,
        }
    }
    setter!(
        percent_done = bool,
        amount_done = bool,
        done_style = Style,
        waiting_style = Style,
        base_style = Style,
        supplementary_newline = bool,
        current = T,
        done_char = char,
        header_char = char,
        waiting_char = char,
        rate = Option<Rate>,
        eta = bool,
    );
    pub fn draw(&mut self) {
        assert!(self.current <= self.target);
        let num_done = (self.current * self.visual_len) / self.target;
        print!("\x1b[s");
        print!(
            "{}[{}{}{}{}{}{}]\x1b[0m",
            self.base_style,
            self.done_style,
            self.done_char
                .to_string()
                .as_str()
                .repeat(<T as PrimAs<usize>>::prim_as(num_done).max(1) - 1),
            {
                if <T as PrimAs<usize>>::prim_as(num_done) == 0 {
                    "".to_string()
                } else {
                    self.header_char.to_string()
                }
            },
            self.waiting_style,
            self.waiting_char
                .to_string()
                .as_str()
                .repeat((self.visual_len - num_done).try_into().unwrap()),
            self.base_style
        );
        if self.supplementary_newline {
            println!();
        }
        if self.amount_done {
            print!(" {}/{}", self.current, self.target);
        }
        if self.percent_done {
            print!(
                " {:.2}%",
                (<T as PrimAs<f64>>::prim_as(self.current)
                    / <T as PrimAs<f64>>::prim_as(self.target))
                    * 100.0
            );
        }
        let now = std::time::SystemTime::now();
        if let Some((prev_time, prev_val)) = self.prev {
            assert!(prev_val <= self.current);
            let value_per_second = <T as PrimAs<f64>>::prim_as(self.current - prev_val)
                / (now.duration_since(prev_time).unwrap().as_secs_f64());
            if let Some(rate) = self.rate {
                match rate {
                    Rate::Absolute => {
                        print!(" {value_per_second:.2}/s");
                    }
                    Rate::Bytes => {
                        let (divisor, prefix) = if value_per_second >= 1000000000.0 {
                            // GigaBytes
                            (1000000000.0, 'G')
                        } else if value_per_second >= 1000000.0 {
                            // MegaBytes
                            (1000000.0, 'M')
                        } else if value_per_second >= 1000.0 {
                            // KiloBytes
                            (1000.0, 'K')
                        } else {
                            (1.0, ' ')
                        };
                        print!(" {}{prefix}Bytes", value_per_second / divisor);
                    }
                }
            }
            if self.eta {
                let eta =
                    <T as PrimAs<f64>>::prim_as(self.target - self.current) / value_per_second;
                print!(" eta: {eta:.2}s");
            }
        }
        self.prev = Some((now, self.current));
        std::io::stdout().flush().unwrap();
    }
    pub fn clear(&self) {
        if self.supplementary_newline {
            print!("\r\x1b[2K\x1b[A\x1b[2K");
        } else {
            print!("\r\x1b[2K");
        }
    }
    pub fn set(&mut self, new_val: T) {
        self.clear();
        self.current = new_val;
        self.draw();
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rate {
    Absolute,
    Bytes,
}
impl FromBinary for Rate {
    fn from_binary(binary: &mut dyn std::io::Read) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(match u8::from_binary(binary)? {
            0 => Rate::Absolute,
            1 => Rate::Bytes,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to get Rate from binary",
                ))
            }
        })
    }
}
impl ToBinary for Rate {
    fn to_binary(&self, binary: &mut dyn Write) -> Result<(), std::io::Error> {
        match self {
            Rate::Absolute => 0_u8,
            Rate::Bytes => 1,
        }
        .to_binary(binary)
    }
}
