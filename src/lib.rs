#[cfg(any(debug_assertions, feature = "ai"))]
pub mod ai;
#[cfg(any(debug_assertions, feature = "file_ops"))]
pub mod file_ops;

pub mod mutec;

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

pub struct OnDrop<'a, T> {
    method: &'a dyn Fn(&T),
    input: T
}
impl<'a, T> OnDrop<'a, T> {
    pub fn new(method: &'a dyn Fn(&T), input: T) -> Self {
        OnDrop {
            method,
            input
        }
    }
    pub fn set_input(&mut self, value: T) {
        self.input = value;
    }
}
impl<'a, T> Drop for OnDrop<'a, T> {
    fn drop(&mut self) {
        let input = &self.input;
        (self.method)(input)
    }
}
/// This will NOT be faster than manually
/// written out dimensions of [Vecs](Vec).
/// The only advantage is that the number
/// of dimensions is defined in a const generic.
/// 
/// This uses consistent side lengths. for example
/// if you have a 3 dimensional [Vec], all the lengths
/// in the x axis will be the same, which also applies to
/// the y and z axes, but they do not need to have the same
/// lengths as each other. So x could be 3 long, while
/// z could be 5 long.
pub struct NVec<T, const N: usize> {
    inner: Vec<T>,
    lengths: [usize; N],
}
impl<T, const N: usize> NVec<T, N> {
    /// Creates an empty [NVec].
    /// Eqivalent to [Vec::new].
    pub const fn new() -> Self {
        NVec {
            inner: Vec::new(),
            lengths: [0; N]
        }
    }
    pub fn get_inner(&self) -> &Vec<T> {
        &self.inner
    }
    pub unsafe fn set_inner(&mut self, inner: Vec<T>, lengths: &[usize; N]) {
        self.inner = inner;
        self.lengths = *lengths;
    }
    /// Assumes the correct number of indexes are given
    fn get_index(&self, indexes: &[usize]) -> usize {
        let mut sum: usize = self.lengths.iter().product();
        let mut target: usize = 0;
        for index in 0..N {
            // Each index needs to be multiplied by
            // the length of everything it contains
            sum /= self.lengths[index];
            target += indexes[index]*sum;
        }
        return target
    }
    /// Gets a reference to the value at the given position.
    pub fn get(&self, indexes: &[usize; N]) -> &T {
        &self.inner[
            self.get_index(indexes)
        ]
    }
    /// Same as [get](NVec::get) but without checks that
    /// the correct number of indexes have been given.
    pub unsafe fn get_slice(&self, indexes: &[usize]) -> &T {
        &self.inner[
            self.get_index(indexes)
        ]
    }
    /// Gets a mutable reference to the value at the given position.
    pub fn get_mut(&mut self, indexes: &[usize; N]) -> &mut T {
        // index needs to be defined before we start getting the actual value
        // because otherwise it will be using an immutable reference to
        // self while getting a mutable reference to something owned by self
        let index: usize = self.get_index(indexes);
        return &mut self.inner[index]
    }
    /// Same as [get_mut](NVec::get_mut) but without checking that
    /// the correct number of indexes have been given.
    pub unsafe fn get_slice_mut(&mut self, indexes: &[usize]) -> &mut T {
        let index: usize = self.get_index(indexes);
        return &mut self.inner[index]
    }
    pub fn clear(&mut self) {
        self.inner = Vec::new();
        self.lengths = [0; N];
    }
}
impl<T, const N: usize> Default for NVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const N: usize> std::ops::Index<[usize; N]> for NVec<T, N> {
    type Output = T;
    fn index(&self, index: [usize; N]) -> &Self::Output {
        &self.inner[
            self.get_index(&index)
        ]
    }
}
impl<T, const N: usize> std::ops::IndexMut<[usize; N]> for NVec<T, N> {
    fn index_mut(&mut self, index: [usize; N]) -> &mut Self::Output {
        let true_index: usize = self.get_index(&index);
        &mut self.inner[true_index]
    }
}
impl<T, const N: usize> std::ops::Index<&[usize]> for NVec<T, N> {
    type Output = T;
    fn index(&self, index: &[usize]) -> &Self::Output {
        &self.inner[
            self.get_index(&index)
        ]
    }
}
impl<T, const N: usize> std::ops::IndexMut<&[usize]> for NVec<T, N> {
    fn index_mut(&mut self, index: &[usize]) -> &mut Self::Output {
        let true_index: usize = self.get_index(&index);
        &mut self.inner[true_index]
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
trait FromBinary {
    fn from_binary(binary: &[u8]) -> Self;
}
impl<T: From<Vec<u8>>> FromBinary for T {
    fn from_binary(binary: &[u8]) -> Self {
        Self::from(binary.to_vec())
    }
}
trait ToBinary {
    fn to_binary(self) -> Vec<u8>;
}
impl<T: Into<Vec<u8>>> ToBinary for T {
    fn to_binary(self) -> Vec<u8> {
        self.into()
    }
}