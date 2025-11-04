//! A collection of types, functions, traits, and macros which
//! I found to be helpful and improve my experience while programming

mod from_binary;
pub use from_binary::{Binary, FromBinary, ToBinary};
mod input;
pub use input::{input, Input};
pub mod split;
pub use split::Split;
pub mod numbers;
pub use numbers::*;
pub mod progress_bar;
pub mod random;
pub use progress_bar::ProgressBar;
pub use random::{initialize, random};
pub mod style;
pub use style::{Color, Style};

pub mod prelude {
    pub use crate::{
        assert_pattern, assert_pattern_ne, debug, debug_println, input, Binary, FromBinary, Input,
        Split, ToBinary,
    };
}

/// A version of [println] that uses the same
/// input syntax but only prints when
/// the crate is not compiled with '--release'
/// it is essentially equivalent to
///```
/// #[cfg(debug_assertations)]
/// println!(/*whatever you gave it*/)
///```
/// For more information, see [println]
#[macro_export]
macro_rules! debug_println {
    () => {
        #[cfg(debug_assertions)]
        println!(())
    };
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("[DEBUG] {}", format_args!($($arg)*));
    }
}
/// A version of [assert] that uses
/// patterns to determine if it has
/// passed. Specifically, this takes
/// in an identifier(variable type thing)
/// then sees if it matches the pattern
/// given. If it does, this does nothing.
/// If it doesn't, then it will [panic]
/// with the message you gave it.
#[macro_export]
macro_rules! assert_pattern {
    ($item: ident, $pattern: pat_param) => {
        if let $pattern = $item {}
        else {
            panic!("Item did not match variant");
        }
    };
    ($item: ident, $pattern: pat_param, $($arg:tt)*) => {
        if let $pattern = $item {}
        else {
            panic!("{}", format_args!($($arg)*));
        }
    };
}
/// A version of [assert_ne] that
/// uses pattern matching to determine
/// whether or not to [panic].
///
/// For more information, see [assert_pattern]
#[macro_export]
macro_rules! assert_pattern_ne {
    ($item: ident, $pattern: pat_param) => {
        if let $pattern = $item {
            panic!("Item matched variant");
        }
    };
    ($item: ident, $pattern: pat_param, $($arg:tt)*) => {
        if let $pattern = $item {
            panic!("{}", format_args!($($arg)*));
        }
    };
}
/// A macro which will only run code
/// when the crate is not compiled
/// with '--release'
///
/// It can be used with a block:
///```should_panic
/// # use abes_nice_things::debug;
/// debug!({
///     //Code only ran in debug mode
/// # panic!("Yay!");
/// });
///```
/// Or it can be used with an expression:
///```should_panic
/// # use abes_nice_things::debug;
/// debug!(/*expression*/);
/// # debug!(panic!("I AM HAVING A PANIC ATTACK"));
///```
/// For example:
///```
/// # use abes_nice_things::debug;
/// debug!({
///     println!("Yippy!");
///     // Any additional code you want
/// });
///```
///```
/// # use abes_nice_things::debug;
/// debug!(println!("Yippy!"));
/// //     ^^^^^ can only have one thing
/// //        (note the lack of parenthesis)
///```
#[macro_export]
macro_rules! debug {
    () => {};
    ($debug: stmt) => {
        #[cfg(debug_assertions)]
        $debug;
    };
    ($debug: block) => {
        if cfg!(debug_assertions) {
            $debug;
        }
    };
}
#[macro_export]
macro_rules! setter {
    ($field:ident, $type:ty) => {
        pub const fn $field(&mut self, val: $type) -> &mut Self {
            self.$field = val;
            self
        }
    };
    ($($field:ident = $type:ty,)*) => {
        $(setter!($field, $type);)*
    }
}

/// Encodes 8 [bool]s into a [u8], compressing them to be an eighth the size. I do not recommend doing
/// any modifying operations on the resulting [u8] if you want to get the [bool]s back.
/// ```
/// # use abes_nice_things::{u8_encode, u8_decode};
/// # fn main() {
/// let bools = [true, false, false, false, true, false, true, true];
/// let compressed = u8_encode(bools);
/// # println!("{compressed:8b}");
/// assert_eq!(bools, u8_decode(compressed));
/// # }
/// ```
pub fn u8_encode(data: [bool; 8]) -> u8 {
    let mut out = 0;
    for (index, state) in data.iter().enumerate() {
        if *state {
            out += 1 << index;
        }
    }
    return out;
}
pub fn u8_decode(data: u8) -> [bool; 8] {
    let mut out = [false; 8];
    for index in 0..8 {
        out[index] = (data & (1 << index)) != 0
    }
    return out;
}

pub trait Increment {
    fn increment(&mut self);
}
pub trait Decriment {
    fn decriment(&mut self);
}
impl<T: Integer> Increment for T
where
    <T as TryFrom<i32>>::Error: std::fmt::Debug,
{
    fn increment(&mut self) {
        *self += T::try_from(1).unwrap();
    }
}
impl<T: Integer> Decriment for T
where
    <T as TryFrom<i32>>::Error: std::fmt::Debug,
{
    fn decriment(&mut self) {
        *self -= T::try_from(1).unwrap()
    }
}
