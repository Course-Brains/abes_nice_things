//! A collection of types, functions, traits, and macros which
//! I found to be helpful and improve my experience while programming

mod from_binary;
pub use from_binary::{Binary, FromBinary, ToBinary};
mod input;
pub use input::Input;

pub use abes_nice_procs::{method, FromBinary, ToBinary};

pub mod prelude {
    pub use crate::{
        assert_pattern, assert_pattern_ne, debug, debug_println, method, Binary, FromBinary,
        ToBinary,
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
