//! A collection of types, functions, traits, and macros which
//! I found to be helpful and improve my experience while programming

mod from_binary;
pub use from_binary::{Binary, FromBinary, ToBinary};
mod input;
pub use input::{Input, input};
pub mod split;
pub use split::Split;

pub mod prelude {
    pub use crate::{
        assert_pattern, assert_pattern_ne, debug, debug_println, Binary, FromBinary,
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
pub fn manual_writer<W: std::io::Write>(mut write: W) -> std::io::Result<()> {
    loop {
        match <Input>::new()
            .msg("What type do you want to write?")
            .cond(&|string| {
                match string.as_str() {
                    "u8" => Ok(true),
                    "u16" => Ok(true),
                    "u32" => Ok(true),
                    "u64" => Ok(true),
                    "u128" => Ok(true),
                    "usize"|"us" => Ok(true),
                    "i8" => Ok(true),
                    "i16" => Ok(true),
                    "i32" => Ok(true),
                    "i64" => Ok(true),
                    "i128" => Ok(true),
                    "isize"|"is" => Ok(true),
                    "char"|"ch" => Ok(true),
                    "stop" => Ok(true),
                    "bin" => Ok(true),
                    "bool"|"bl" => Ok(true),
                    _ => Ok(false)
                }
        }).get().unwrap().as_str() {
            "u8" => input().parse::<u8>().unwrap().to_binary(&mut write)?,
            "u16" => input().parse::<u16>().unwrap().to_binary(&mut write)?,
            "u32" => input().parse::<u32>().unwrap().to_binary(&mut write)?,
            "u64" => input().parse::<u64>().unwrap().to_binary(&mut write)?,
            "u128" => input().parse::<u128>().unwrap().to_binary(&mut write)?,
            "usize"|"us" => input().parse::<usize>().unwrap().to_binary(&mut write)?,
            "i8" => input().parse::<i8>().unwrap().to_binary(&mut write)?,
            "i16" => input().parse::<i16>().unwrap().to_binary(&mut write)?,
            "i32" => input().parse::<i32>().unwrap().to_binary(&mut write)?,
            "i64" => input().parse::<i64>().unwrap().to_binary(&mut write)?,
            "i128" => input().parse::<i128>().unwrap().to_binary(&mut write)?,
            "isize"|"is" => input().parse::<isize>().unwrap().to_binary(&mut write)?,
            "char"|"ch" => {
                let binding = input();
                let mut input = binding.chars();
                let first = input.next().unwrap();
                if first != '\\' {
                    // Normal character
                    first.to_binary(&mut write)?;
                    continue;
                }
                // Escaped character
                let second = input.next().unwrap();
                match second {
                    'n' => '\n'.to_binary(&mut write)?,
                    _ => eprintln!("Invalid escaped character: {second}")
                }
            }
            "bin" => {
                let mut out = 0_u8;
                for (index, ch) in input().chars().rev().enumerate() {
                    match ch {
                        '1' => out += 1<<index,
                        '0'|'_' => {}
                        _ => {
                            eprintln!("Failed due to invalid binary");
                            break;
                        }
                    }
                }
                out.to_binary(&mut write)?;
            }
            "bool"|"bl" => input().parse::<bool>().unwrap().to_binary(&mut write)?,
            "stop" => break,
            _ => unreachable!("Fucky wucky!")
        }
    }
    Ok(())
}
pub fn manual_reader<R: std::io::Read>(mut read: R) -> std::io::Result<()> {
    loop {
        match <Input>::new()
            .msg("What type are you reading")
            .cond(&|string| {
                match string.as_str() {
                    "u8" => Ok(true),
                    "u16" => Ok(true),
                    "u32" => Ok(true),
                    "u64" => Ok(true),
                    "u128" => Ok(true),
                    "usize"|"us" => Ok(true),
                    "i8" => Ok(true),
                    "i16" => Ok(true),
                    "i32" => Ok(true),
                    "i64" => Ok(true),
                    "i128" => Ok(true),
                    "isize"|"is" => Ok(true),
                    "char"|"ch" => Ok(true),
                    "stop" => Ok(true),
                    "bin" => Ok(true),
                    "bool"|"bl" => Ok(true),
                    _ => Ok(false)
                }
            })
        .get().unwrap().as_str() {
            "u8" => println!("{}", u8::from_binary(&mut read)?),
            "u16" => println!("{}", u16::from_binary(&mut read)?),
            "u32" => println!("{}", u32::from_binary(&mut read)?),
            "u64" => println!("{}", u64::from_binary(&mut read)?),
            "u128" => println!("{}", u128::from_binary(&mut read)?),
            "usize"|"us" => println!("{}", usize::from_binary(&mut read)?),
            "i8" => println!("{}", i8::from_binary(&mut read)?),
            "i16" => println!("{}", i16::from_binary(&mut read)?),
            "i32" => println!("{}", i32::from_binary(&mut read)?),
            "i64" => println!("{}", i64::from_binary(&mut read)?),
            "i128" => println!("{}", i128::from_binary(&mut read)?),
            "isize"|"is" => println!("{}", isize::from_binary(&mut read)?),
            "bin" => {
                let byte = u8::from_binary(&mut read)?;
                print!("{}", "0".repeat(byte.leading_zeros() as usize));
                println!("{byte:b}");
            }
            "char" => {
                let ch = char::from_binary(&mut read)?;
                match ch {
                    '\n' => println!("newline"),
                    ' ' => println!("<space>"),
                    _ => println!("{ch}"),
                }
            }
            "bool"|"bl" => println!("{}", bool::from_binary(&mut read)?),
            "stop" => break,
            _ => unreachable!("Fucky wucky!")
        }
    }
    Ok(())
}
