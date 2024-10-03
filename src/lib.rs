pub mod mutec;
pub mod as_from;
pub use as_from::{AsFrom, AsInto, AsTryFrom, AsTryInto};

use rand::distributions::uniform::{SampleRange, SampleUniform};
use serde::{Serialize, Deserialize};
use std::{
    hash::{Hash, Hasher},
    io::stdin,
    ops::{Range, RangeBounds, RangeInclusive},
    sync::{Mutex, MutexGuard},
    fmt::Debug
};

pub mod prelude {
    pub use crate::{
        assert_pattern,
        assert_pattern_ne,
        debug,
        debug_println,
        input,
        AsFrom,
        AsInto,
        AsTryFrom,
        AsTryInto
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

/// A version of [OnceLock](std::sync::OnceLock)
/// which has the method used be determined at creation and consistent.
/// Main benefit is that you won't have to type out the method multiple
/// times and risk mistakes.
pub struct OnceLockMethod<'a, T> {
    inner: Mutex<Option<T>>,
    method: &'a (dyn Fn() -> T + Sync),
}
impl<'a, T> OnceLockMethod<'a, T> {
    /// Creates the instance, but does NOT
    /// initialize the data.
    pub const fn new(method: &'a (impl Fn() -> T + Sync)) -> OnceLockMethod<'a, T> {
        return OnceLockMethod {
            inner: Mutex::new(None),
            method,
        };
    }
    /// Runs the method to set the data
    /// even if it already has been set.
    /// Also, this is blocking.
    pub fn init(&self) {
        *self.inner.lock().unwrap() = Some((self.method)());
    }
    /// Gets the data,
    /// or if there is no data yet,
    /// returns None.
    pub fn get(&self) -> MutexGuard<'_, Option<T>> {
        return self.inner.lock().unwrap();
    }
    /// If it has not already been initalized,
    /// it creates the data.
    /// Then it gets the data.
    /// This is also blocking.
    pub fn get_or_init(&self) -> MutexGuard<'_, Option<T>> {
        if self.is_uninit() {
            self.init();
        }
        return self.get();
    }
    /// returns whether or not this has been initialized.
    pub fn is_init(&self) -> bool {
        if self.inner.lock().unwrap().is_some() {
            return true;
        }
        return false;
    }
    /// returns if this has NOT been initialized.
    pub fn is_uninit(&self) -> bool {
        if self.inner.lock().unwrap().is_none() {
            return true;
        }
        return false;
    }
    /// Sets the data to something no matter what
    /// its state was before.
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
    pub fn to_vec(&self) -> &Vec<T> {
        &self.inner
    }
    pub unsafe fn set_inner(&mut self, inner: Vec<T>, lengths: &[usize; N]) {
        self.inner = inner;
        self.lengths = *lengths;
    }
    /// Assumes the correct number of indexes are given
    fn get_index(&self, indexes: &[usize]) -> usize {
        assert_eq!(
            indexes.len(), self.lengths.len(),
            "Incorrect number of indexes given:\nexpected: {}, got:{}", self.lengths.len(), indexes.len());
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
impl<T: rand::Fill, const N: usize> rand::Fill for NVec<T, N> {
    fn try_fill<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), rand::Error> {
        for item in self.inner.iter_mut() {
            rng.try_fill(item)?
        }
        Ok(())
    }
}
pub struct ArgChecks<'a, const N: usize> {
    checks: [ArgCheck<'a> ; N]
}
impl<'a, const N: usize> ArgChecks<'a, N> {
    pub const fn new(checks: [ArgCheck<'a>; N]) -> Self {
        ArgChecks {
            checks
        }
    }
    pub fn check(&self) {
        self.check_with(&mut std::env::args())
    }
    pub fn check_with(&self, args: &mut std::env::Args) {
        while let Some(arg) = args.next() {
            if let Some(check) = self.contains(arg) {
                match check.args {
                    Some(num_args) => {
                        let mut sub_args = Vec::with_capacity(num_args);
                        for _ in 0..num_args {
                            // 7 layers of indentation pain
                            sub_args.push(args.next().unwrap())
                        }
                        (check.run)(sub_args)
                    }
                    None => {
                        (check.run)(Vec::new())
                    }
                }
            }
        }
    }
    fn contains(&self, other: String) -> Option<&ArgCheck> {
        for check in self.checks.iter() {
            if check.trigger == other {
                return Some(check)
            }
        }
        None
    }
}
pub struct ArgCheck<'a> {
    pub trigger: &'a str,
    pub args: Option<usize>,
    pub run: &'a dyn Fn(Vec<String>),
}
/// An async adjacent way to generate things using threads.
pub struct ThreadInit<T: std::marker::Send + Debug + 'static> {
    data: std::sync::OnceLock<T>,
    handle: Option<std::thread::JoinHandle<T>>,
    method: Box<dyn Fn() -> T + std::marker::Send>
}
impl<T: std::marker::Send + Debug + 'static> ThreadInit<T> {
    /// Creates the instance and starts the thread's operations.
    pub fn new<C: Fn() -> T + std::marker::Send + Clone + 'static>(creator:&'static C) -> Self {
        ThreadInit {
            data: std::sync::OnceLock::new(),
            handle: Some(std::thread::spawn(creator.clone())),
            method: Box::new(creator.clone())
        }
    }
    /// Joins with the thread and gets the data returned.
    /// If it already happened then it will just give a stored value.
    /// Any error returned is from the threads joining.
    pub fn get(&mut self) -> Result<&T, Box<dyn std::any::Any + std::marker::Send>> {
        // If we already have the value, then we cam just return it
        if let Some(data) = self.data.get() {
            return Ok(data)
        }
        // When data gets initialized, the handle gets consumed,
        // and at this point, that hasn't happened.
        // Meaning that there is no situation where at this point,
        // data has already been set, or the handle has been consumed
        self.data.set(
            self.handle.take().expect("Something has gone very very wrong")
            .join()?
        ).expect("Something has gone very very wrong");
        // If we can't get the value we literally just set then I don't even know anymore
        return Ok(self.data.get().expect("Something has gone very very wrong"));
    }
    /// Removes the data from the thread(if there was any),
    /// sets the thread to regenerate the data,
    /// and returns the data that there, if there was any.
    pub fn reset(&mut self) -> Option<T> where  dyn Fn() -> T + std::marker::Send: Clone{
        let out = self.data.take();
        self.handle = Some(std::thread::spawn((*self.method).clone()));
        return out
    }
    /// Consumes the instance and if the data has already been
    /// generated, it returns that. But if it hasn't been
    /// then it finishes generation then does it.
    /// As such, until it finishes generation,
    /// it blocks the current thread.
    pub fn unwrap(mut self) -> T {
        if let Some(data) = self.data.take() {
            return data
        }
        return self.handle.unwrap().join().unwrap()
    }
}
use std::sync::mpsc::{Sender, Receiver, channel, SendError, RecvError};
pub struct Transceiver<T> {
    tx: Sender<T>,
    rx: Receiver<T>,
}
impl<T> Transceiver<T> {
    pub fn new() -> (Self, Self) {
        let (tx1, rx2) = channel::<T>();
        let (tx2, rx1) = channel::<T>();
        return (
            Transceiver {
                tx: tx1,
                rx: rx1
            },
            Transceiver {
                tx: tx2,
                rx: rx2
            }
        )
    }
    pub fn send(&self, data: T) -> Result<(), SendError<T>> {
        self.tx.send(data)
    }
    pub fn recv(&self) -> Result<T, RecvError> {
        self.rx.recv()
    }
    pub fn call(&self, data: T) -> Result<T, TransError<T>> {
        self.tx.send(data)?;
        Ok(self.rx.recv()?)
    }
}
#[derive(Debug)]
pub enum TransError<T> {
    Send(SendError<T>),
    Recv(RecvError)
}
impl<T> From<SendError<T>> for TransError<T> {
    fn from(value: SendError<T>) -> Self {
        TransError::Send(value)
    }
}
impl<T> From<RecvError> for TransError<T> {
    fn from(value: RecvError) -> Self {
        TransError::Recv(value)
    }
}
/// A concrete type for storing the range types while Sized.
/// Currently only has [Range] and [RangeInclusive]
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
pub enum Either<T, U> {
    T(T),
    U(U)
}
impl<T, U> Either<T, U> {
    pub fn new_t(t: T) -> Self {
        Either::T(t)
    }
    pub fn new_u(u: U) -> Self {
        Either::U(u)
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
pub trait FromBinary {
    fn from_binary(binary: &[u8]) -> Self;
}
pub trait FromBinarySized where Self: FromBinary {
    const LEN: usize;
    /*fn read_next(mut source: impl Read) -> Self where Self: Sized {
        let mut buf = [0u8; Self::LEN];
        source.read_exact(&mut buf);
        Self::from_binary(&buf)
    }*/
}
impl FromBinary for u8 {
    fn from_binary(binary: &[u8]) -> Self {
        binary[0]
    }
}
impl FromBinarySized for u8 {
    const LEN: usize = 1;
}
impl FromBinary for u16 {
    fn from_binary(binary: &[u8]) -> Self {
        u16::from_le_bytes([binary[0], binary[1]])
    }
}
pub trait ToBinary {
    fn to_binary(self) -> Vec<u8>;
}
impl<T: Into<Vec<u8>>> ToBinary for T {
    fn to_binary(self) -> Vec<u8> {
        self.into()
    }
}
#[cfg(test)]
mod tests {
    mod thread_init {
        use super::super::ThreadInit;
        #[test]
        fn new() {
            let _a = ThreadInit::new(&|| {5});
        }
        #[test]
        fn normal() {
            let mut init = ThreadInit::new(&|| {"uisx".to_string()});
            assert_eq!(init.get().unwrap(), "uisx");
        }
        #[test]
        fn unwrap_gen() {
            let init = ThreadInit::new(&|| {"shrug"});
            assert_eq!(init.unwrap(), "shrug");
        }
        #[test]
        fn unwrap_pre_gen() {
            let mut init = ThreadInit::new(&|| {15});
            assert_eq!(*init.get().unwrap(), 15);
            assert_eq!(init.unwrap(), 15);
        }
    }
}