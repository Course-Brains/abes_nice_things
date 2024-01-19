#[cfg(any(debug_assertions, feature = "ai"))]
pub mod ai;
#[cfg(any(debug_assertions, feature = "file_ops"))]
pub mod file_ops;

use rand::distributions::uniform::{SampleRange, SampleUniform};
use serde::{Serialize, Deserialize};
use std::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    hash::{Hash, Hasher},
    io::{stdin, stdout, Write},
    ops::{Deref, DerefMut, Range, RangeBounds, RangeInclusive},
    sync::{Mutex, MutexGuard},
};

pub mod prelude {
    pub use crate::{
        assert_pattern, assert_pattern_ne, debug, debug_println, input, PartialIterOrd,
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
/// Stores a type and a weight associated with it,
/// it also lets you do operations based on the weight.
/// Some traits are passed down to the internal value,
/// like Iterator.
#[derive(Eq, Debug)]
pub struct Weight<T> {
    inner: T,
    pub weight: usize,
}
impl<T> Weight<T> {
    pub const DEFAULT_WEIGHT: usize = 0;
    pub const fn new(inner: T) -> Self {
        return Weight {
            inner,
            weight: Self::DEFAULT_WEIGHT,
        };
    }
    // Internal operations
    pub const fn as_ref(&self) -> Weight<&T> {
        return Weight {
            inner: &self.inner,
            weight: self.weight,
        };
    }
    pub fn as_mut(&mut self) -> Weight<&mut T> {
        return Weight {
            inner: &mut self.inner,
            weight: self.weight,
        };
    }
    pub fn as_deref(&self) -> Weight<&T::Target>
    where
        T: Deref,
    {
        return Weight {
            inner: self.inner.deref(),
            weight: self.weight,
        };
    }
    pub fn as_deref_mut(&mut self) -> Weight<&mut T::Target>
    where
        T: DerefMut,
    {
        return Weight {
            inner: self.inner.deref_mut(),
            weight: self.weight,
        };
    }
    // For comparing Weight and usize
    pub const fn lt_direct(&self, other: usize) -> bool {
        return self.weight < other;
    }
    pub const fn le_direct(&self, other: usize) -> bool {
        return self.weight <= other;
    }
    pub const fn gt_direct(&self, other: usize) -> bool {
        return self.weight > other;
    }
    pub const fn ge_direct(&self, other: usize) -> bool {
        return self.weight >= other;
    }
    pub const fn partial_cmp_direct(&self, other: usize) -> Option<Ordering> {
        if self.weight > other {
            return Some(Ordering::Greater);
        }
        if self.weight < other {
            return Some(Ordering::Less);
        }
        if self.weight == other {
            return Some(Ordering::Equal);
        }
        return None;
    }
    pub const fn cmp_direct(&self, other: usize) -> Ordering {
        if self.weight > other {
            return Ordering::Greater;
        }
        if self.weight < other {
            return Ordering::Less;
        }
        return Ordering::Equal;
    }
    // Ord methods if T doesn't impl Eq
    pub fn max(self, other: Self) -> Self {
        if self > other {
            return self;
        }
        return other;
    }
    pub fn min(self, other: Self) -> Self {
        if self <= other {
            return self;
        }
        return other;
    }
    pub fn clamp(self, min: Self, max: Self) -> Self {
        assert!(min <= max);
        if self < min {
            return min;
        }
        if self > max {
            return max;
        }
        return self;
    }
    pub fn cmp(&self, other: &Self) -> Ordering {
        if self < other {
            return Ordering::Less;
        }
        if self > other {
            return Ordering::Greater;
        }
        return Ordering::Equal;
    }
}
impl<T> PartialEq for Weight<T> {
    fn eq(&self, other: &Self) -> bool {
        return self.weight == other.weight;
    }
    fn ne(&self, other: &Self) -> bool {
        return self.weight != other.weight;
    }
}
impl<T> PartialOrd for Weight<T> {
    fn lt(&self, other: &Self) -> bool {
        return self.weight < other.weight;
    }
    fn le(&self, other: &Self) -> bool {
        return self.weight <= other.weight;
    }
    fn gt(&self, other: &Self) -> bool {
        return self.weight > other.weight;
    }
    fn ge(&self, other: &Self) -> bool {
        return self.weight >= other.weight;
    }
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self < other {
            return Some(Ordering::Less);
        }
        if self > other {
            return Some(Ordering::Greater);
        }
        if self == other {
            return Some(Ordering::Equal);
        }
        return None;
    }
}
impl<T: Eq> Ord for Weight<T> {
    fn max(self, other: Self) -> Self {
        if self > other {
            return self;
        }
        return other;
    }
    fn min(self, other: Self) -> Self {
        if self <= other {
            return self;
        }
        return other;
    }
    fn clamp(self, min: Self, max: Self) -> Self {
        assert!(min <= max);
        if self < min {
            return min;
        }
        if self > max {
            return max;
        }
        return self;
    }
    fn cmp(&self, other: &Self) -> Ordering {
        if self < other {
            return Ordering::Less;
        }
        if self > other {
            return Ordering::Greater;
        }
        return Ordering::Equal;
    }
}
impl<T: Default> Default for Weight<T> {
    fn default() -> Self {
        return Weight {
            inner: Default::default(),
            weight: Default::default(),
        };
    }
}
impl<T: Clone> Clone for Weight<T> {
    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
    fn clone(&self) -> Self {
        return Weight {
            inner: self.inner.clone(),
            weight: self.weight.clone(),
        };
    }
}
impl<T: Copy> Copy for Weight<T> {}
impl<T> Borrow<T> for Weight<T> {
    fn borrow(&self) -> &T {
        return &self.inner;
    }
}
impl<T> BorrowMut<T> for Weight<T> {
    fn borrow_mut(&mut self) -> &mut T {
        return &mut self.inner;
    }
}
impl<T: Iterator> Iterator for Weight<T> {
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        return self.inner.next();
    }
}
impl<T: DoubleEndedIterator> DoubleEndedIterator for Weight<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}
impl<T: ExactSizeIterator> ExactSizeIterator for Weight<T> {}
impl<T: Hash> Hash for Weight<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
        self.weight.hash(state);
    }
}
impl<T> Unpin for Weight<T> {}

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
        panic!("Attempted to unwrap to range on non-range value")
    }
    pub fn unwrap_inclusive(self) -> RangeInclusive<T> {
        if let Ranges::Inclusive(range) = self {
            return range;
        }
        panic!("Attampted to unwrap to inclusive range on non inclusive range value")
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
    stdout().flush().unwrap();
    stdin().read_line(&mut string).unwrap();
    if let Some('\n') = string.chars().next_back() {
        string.pop();
    } else if let Some('\r') = string.chars().next_back() {
        string.pop();
    }
    return string;
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

pub trait PartialIterOrd
where
    Self: IntoIterator + Sized,
    <Self as IntoIterator>::Item: PartialOrd,
{
    fn index_max(self) -> usize {
        let mut iter: <Self as IntoIterator>::IntoIter = self.into_iter();
        let mut largest: <Self as IntoIterator>::Item = iter.nth(0).unwrap();
        let mut largest_index: usize = 0;
        for (index, value) in iter.enumerate() {
            if value > largest {
                largest = value;
                largest_index = index
            }
        }
        return largest_index;
    }
    fn index_min(self) -> usize {
        let mut iter: <Self as IntoIterator>::IntoIter = self.into_iter();
        let mut smallest: <Self as IntoIterator>::Item = iter.nth(0).unwrap();
        let mut smallest_index: usize = 0;
        for (index, value) in iter.enumerate() {
            if value < smallest {
                smallest = value;
                smallest_index = index
            }
        }
        return smallest_index;
    }
    fn index_max_by(
        self,
        method: impl Fn(&<Self as IntoIterator>::Item, &<Self as IntoIterator>::Item) -> Ordering,
    ) -> usize {
        let mut iter: <Self as IntoIterator>::IntoIter = self.into_iter();
        let mut largest: <Self as IntoIterator>::Item = iter.nth(0).unwrap();
        let mut largest_index: usize = 0;
        for (index, value) in iter.enumerate() {
            if let Ordering::Greater = method(&value, &largest) {
                largest = value;
                largest_index = index;
            }
        }
        return largest_index;
    }
    fn index_min_by(
        self,
        method: impl Fn(&<Self as IntoIterator>::Item, &<Self as IntoIterator>::Item) -> Ordering,
    ) -> usize {
        let mut iter: <Self as IntoIterator>::IntoIter = self.into_iter();
        let mut smallest: <Self as IntoIterator>::Item = iter.nth(0).unwrap();
        let mut smallest_index: usize = 0;
        for (index, value) in iter.enumerate() {
            if let Ordering::Less = method(&value, &smallest) {
                smallest = value;
                smallest_index = index;
            }
        }
        return smallest_index;
    }
    fn partial_max(self) -> <Self as IntoIterator>::Item {
        let mut iter: <Self as IntoIterator>::IntoIter = self.into_iter();
        let mut largest: <Self as IntoIterator>::Item = iter.nth(0).unwrap();
        for value in iter {
            if value > largest {
                largest = value
            }
        }
        return largest;
    }
    fn partial_min(self) -> <Self as IntoIterator>::Item {
        let mut iter: <Self as IntoIterator>::IntoIter = self.into_iter();
        let mut smallest: <Self as IntoIterator>::Item = iter.nth(0).unwrap();
        for value in iter {
            if value < smallest {
                smallest = value
            }
        }
        return smallest;
    }
}
impl<I: IntoIterator> PartialIterOrd for I where <I as IntoIterator>::Item: PartialOrd {}