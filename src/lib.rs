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
#[clippy::format_args]
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
    ($($token:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::expand!($($token)*);
    }
}
/// Creates a method which assigns the value of the given field and returns a mutable reference to
/// Self once done.
///
/// This has a use case compared to just setting the field public as it allows for setting the
/// value using chainable methods, much like [OpenOptions]
///
/// It does have to go inside an impl statement for the type you want to create methods for, and
/// you do need to give it the type of the field.
/// ```ignore
/// # use abes_nice_things::setter;
/// #[derive(Default)]
/// struct Example {
///     a: usize,
///     b: String,
/// }
/// impl Example {
///     setter!(
///         a = usize,
///         b = String
///     );
/// }
/// // Is equivalent to
/// impl Example {
///     setter!(a, usize);
///     setter!(b, String);
/// }
/// // Is equivalent to
/// impl Example {
///     pub const fn a(&mut self, val: usize) -> &mut Self {
///         self.a = val;
///         self
///     }
///     pub const fn b(&mut self, val: String) -> &mut Self {
///         self.b = val;
///         self
///     }
/// }
///
/// // All three allow
/// fn main() {
///     Example::default()
///         .a(5)
///         .b("Hello".to_string());
///
///     // Which would otherwise require
///     let mut example = Example::default();
///     example.a = 5;
///     example.b = "Hello".to_string();
/// }
/// ```
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
/// A nothing macro which just takes in tokens and spits them back out unmodified. But it is useful
/// for making macros that will use compile flags as you can ensure that all tokens are affected by
/// the flag by having the tokens pass through this and using the flag on this aswell.
///
/// For example, this is how [debug] works
/// ```
/// # use abes_nice_things::expand;
/// macro_rules! debug {
///     ($($token:tt)*) => {
///         #[cfg(debug_assertions)]
///         $crate::expand!($($token)*);
///     }
/// }
/// ```
#[macro_export]
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
/// let mut value = 5;
/// require_debug!(
///     println!("Hello!");
///     value += 16;
///     println!("I just modified something!");
/// );
/// # }
/// ```
#[macro_export]
macro_rules! require_debug {
    ($($tokens:tt)*) => {
        #[cfg(debug_assertions)]
        $crate::expand!($($tokens)*);
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
        $crate::expand!($($tokens:tt)*);
    };
}
/// Only keep the given code on unix family targets.
///
/// If the target this is compiled for is considered a part of the "unix family" by rust, then the
/// code will be kept.
///
/// Put whatever you want in this, and if the compiler complains then do what it says it has a
/// hostage.
#[macro_export]
macro_rules! unix {
    ($($tokens:tt)*) => {
        #[cfg(target_family = "unix")]
        $crate::expand!($($tokens)*);
    };
}
