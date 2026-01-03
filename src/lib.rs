//! A collection of types, functions, traits, and macros which
//! I found to be helpful and improve my experience while programming

mod from_binary;
pub use from_binary::{Binary, FromBinary, ToBinary};
mod input;
pub use input::{Input, input};
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
        Binary, FromBinary, Input, Split, ToBinary, assert_pattern, assert_pattern_ne, debug,
        debug_println, input,
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
// I don't know why the linter thinks this isn't used, it literally is
#[allow(unused_macros)]
macro_rules! expand {
    ($($token:tt)*) => {
        $($token)*
    }
}
/// A macro which will pass through whatever tokens you give it on debug, but will fail to compile
/// on release.
///
/// Because this passes any tokens given to it, you can put debugging placeholder values in it and
/// you will not be able to compile for release until you replace them.
///
/// ```
/// # use abes_nice_things::require_debug;
/// # fn main() {
///
/// # }
/// ```
#[macro_export]
macro_rules! require_debug {
    ($($tokens:tt)*) => {
        #[cfg(debug_assertions)]
        expand!($($tokens)*);
        #[cfg(not(debug_assertions))]
        compile_error!("Attempted to compile debug only code in release");
    };
}
/// Only keep the given code on windows family targets.
///
/// This will keep the given code if the os it is compiled for is considered a part of the "windows
/// family" by rust as defined in the reference.
///
/// You can put any code in this, but it does follow the usual restrictions for macros.
#[macro_export]
macro_rules! windows {
    ($($tokens:tt)*) => {
        #[cfg(target_family = "windows")]
        expand!($($tokens:tt)*);
    };
}
/// Only keep the given code on uxit family targets.
///
/// If the target this is compiled for is considered a part of the "unix" family by rust, then the
/// code will be kept.
///
///
#[macro_export]
macro_rules! unix {
    ($($tokens:tt)*) => {
        #[cfg(target_family = "unix")]
        expand!($($tokens)*);
    };
}
