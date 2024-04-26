#[cfg(any(debug_assertions, feature = "ai"))]
pub mod ai;
#[cfg(any(debug_assertions, feature = "file_ops"))]
pub mod file_ops;

pub mod nvec;

use rand::distributions::uniform::{SampleRange, SampleUniform};
use serde::{Serialize, Deserialize};
use std::{
    hash::{Hash, Hasher},
    io::stdin,
    ops::{Range, RangeBounds, RangeInclusive},
    sync::{Mutex, MutexGuard},
};

pub mod prelude {
    pub use crate::{
        assert_pattern,
        assert_pattern_ne,
        debug,
        debug_println,
        input,
    };
}

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
#[macro_export]
macro_rules! debug {
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

pub struct OnceLockMethod<'a, T> {
    inner: Mutex<Option<T>>,
    method: &'a (dyn Fn() -> T + Sync),
}
impl<'a, T> OnceLockMethod<'a, T> {
    pub const fn new(method: &'a (impl Fn() -> T + Sync)) -> OnceLockMethod<'a, T> {
        return OnceLockMethod {
            inner: Mutex::new(None),
            method,
        };
    }
    pub fn init(&self) {
        *self.inner.lock().unwrap() = Some((self.method)());
    }
    pub fn get(&self) -> MutexGuard<'_, Option<T>> {
        return self.inner.lock().unwrap();
    }
    pub fn get_or_init(&self) -> MutexGuard<'_, Option<T>> {
        if self.is_uninit() {
            self.init();
        }
        return self.get();
    }
    pub fn is_init(&self) -> bool {
        if self.inner.lock().unwrap().is_some() {
            return true;
        }
        return false;
    }
    pub fn is_uninit(&self) -> bool {
        if self.inner.lock().unwrap().is_none() {
            return true;
        }
        return false;
    }
    pub unsafe fn set(&self, value: T) {
        *self.inner.lock().unwrap() = Some(value)
    }
}

/// A concrete type for storing the range types while Sized.
#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub enum Ranges<T> {
    Range(Range<T>),
    Inclusive(RangeInclusive<T>),
}
impl<T> Ranges<T> {
    pub fn unwrap_range(self) -> Range<T> {
        if let Ranges::Range(range) = self {
            return range;
        }
        panic!("Attempted to unwrap to range on non-range value ")
    }
    pub fn unwrap_inclusive(self) -> RangeInclusive<T> {
        if let Ranges::Inclusive(range) = self {
            return range;
        }
        panic!("Attempted to unwrap to inclusive range on non inclusive range value")
    }
}
impl<T: Default> Default for Ranges<T> {
    fn default() -> Self {
        return Ranges::Range(Default::default());
    }
}
impl<T: Clone> Clone for Ranges<T> {
    fn clone(&self) -> Self {
        match self {
            Self::Range(range) => Ranges::Range(range.clone()),
            Self::Inclusive(range) => Ranges::Inclusive(range.clone()),
        }
    }
}
impl<T: Hash> Hash for Ranges<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}
impl<T: SampleUniform + PartialOrd> SampleRange<T> for Ranges<T> {
    fn sample_single<R: rand::prelude::RngCore + ?Sized>(self, rng: &mut R) -> T {
        match self {
            Ranges::Range(range) => return range.sample_single(rng),
            Ranges::Inclusive(range) => return range.sample_single(rng),
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            Ranges::Range(range) => return range.is_empty(),
            Ranges::Inclusive(range) => return range.is_empty(),
        }
    }
}
impl<T> From<Range<T>> for Ranges<T> {
    fn from(value: Range<T>) -> Self {
        return Ranges::Range(value);
    }
}
impl<T> From<RangeInclusive<T>> for Ranges<T> {
    fn from(value: RangeInclusive<T>) -> Self {
        return Ranges::Inclusive(value);
    }
}
impl<T> RangeBounds<T> for Ranges<T> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        match self {
            Ranges::Range(range) => return range.start_bound(),
            Ranges::Inclusive(range) => return range.start_bound(),
        }
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        match self {
            Ranges::Range(range) => return range.end_bound(),
            Ranges::Inclusive(range) => return range.end_bound(),
        }
    }
}

pub fn gen_check<T>(gen: impl Fn() -> T, check: impl Fn(&T) -> bool) -> T {
    loop {
        let value: T = gen();
        if check(&value) {
            return value;
        }
    }
}
pub fn unwrap_none<T>(input: &Option<T>, message: &str) {
    if let Some(_) = input {
        panic!("{message}")
    }
}
pub fn input() -> String {
    let mut string: String = String::new();
    stdin().read_line(&mut string).unwrap();
    if let Some('\n') = string.chars().next_back() {
        string.pop();
    } else if let Some('\r') = string.chars().next_back() {
        string.pop();
    }
    string
}
pub fn input_cond(cond: impl Fn(&String) -> Result<bool, String>) -> Result<String, String> {
    loop {
        let input: String = input();
        match cond(&input) {
            Ok(value) => {
                if value {
                    return Ok(input);
                }
            }
            Err(error) => return Err(error),
        }
    }
}
pub fn input_allow(allow: &[String]) -> String {
    loop {
        let input = input();
        if allow.contains(&input) {
            return input
        }
    }
}
pub fn input_allow_msg(allow: &[String], msg: &str) -> String {
    loop {
        println!("{msg}");
        let input = input();
        if allow.contains(&input) {
            return input
        }
    }
}
pub fn input_yn(msg: &str) -> bool {
    let mut value;
    loop {
        println!("{msg}");
        value = input();
        if value == "y" {
            return true
        }
        if value == "n" {
            return false
        }
    }
}