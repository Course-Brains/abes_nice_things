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
    waiting_style: Style,
    base_style: Style,
    supplementary_newline: bool,
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
    );
    pub fn draw(&self) {
        assert!(self.current <= self.target);
        let num_done = (self.current * self.visual_len) / self.target;
        print!(
            "{}[{}{}{}{}{}]",
            self.base_style,
            self.done_style,
            "#".repeat(num_done.try_into().unwrap()),
            self.waiting_style,
            "-".repeat((self.visual_len - num_done).try_into().unwrap()),
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
        std::io::stdout().flush().unwrap();
    }
    pub fn clear(&self) {
        if self.supplementary_newline {
            print!("\r\x1b[2K\x1b[A\x1b[2K");
        }
    }
    pub fn set(&mut self, new_val: T) {
        self.clear();
        self.current = new_val;
        self.draw();
    }
}
impl<T: UnsignedInteger> FromBinary for ProgressBar<T> {
    fn from_binary(binary: &mut dyn std::io::Read) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(ProgressBar {
            current: T::from_binary(binary)?,
            target: T::from_binary(binary)?,
            visual_len: T::from_binary(binary)?,
            percent_done: bool::from_binary(binary)?,
            amount_done: bool::from_binary(binary)?,
            done_style: Style::from_binary(binary)?,
            waiting_style: Style::from_binary(binary)?,
            base_style: Style::from_binary(binary)?,
            supplementary_newline: bool::from_binary(binary)?,
        })
    }
}
impl<T: UnsignedInteger> ToBinary for ProgressBar<T> {
    fn to_binary(&self, binary: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        macro_rules! helper {
            ($($field:ident)*) => {
                $(self.$field.to_binary(binary)?;)*
            }
        }
        helper!(current target visual_len percent_done amount_done done_style waiting_style base_style supplementary_newline);
        Ok(())
    }
}
