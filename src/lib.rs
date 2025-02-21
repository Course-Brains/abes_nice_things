//! A collection of types, functions, traits, and macros which
//! I found to be helpful and improve my experience while programming

pub mod as_from;
pub use as_from::{AsFrom, AsInto, AsTryFrom, AsTryInto};
pub mod from_binary;
pub use from_binary::{FromBinary,ToBinary, Binary};
pub mod item;
pub use item::Item;

use std::{
    io::stdin,
    sync::{Mutex, MutexGuard},
};

pub use abes_nice_procs::{FromBinary, ToBinary, method};

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
        AsTryInto,
        method,
        FromBinary,
        ToBinary,
        Binary
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
    input: T,
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
/// A type to run code when the thread panics.
/// In order to actually run the code,
/// the instance of this must exist when the thread panics.
/// aka. it needs to not have been dropped.
/// To make that easier, I suggest you put it in
/// the main section of your thread, or put it in main.
/// However, you can also put it in a function to handle panics
/// specifically inside that function.
/// Or you can drop it to remove it when you no longer need it.
pub struct OnPanic<'a, T> {
    method: &'a dyn Fn(&T),
    input: T,
}
impl<'a, T> OnPanic<'a, T> {
    pub fn new(method: &'a dyn Fn(&T), input: T) -> Self {
        OnPanic {
            method,
            input
        }
    }
    /// Sets the input to the method and returns the previous value
    pub fn set_input(&mut self, input: T) -> T {
        std::mem::replace(&mut self.input, input)
    }
    pub fn get_input(&self) -> &T {
        &self.input
    }
    pub fn set_method(&mut self, method: &'a dyn Fn(&T)) {
        self.method = method;
    }
}
impl<'a, T> Drop for OnPanic<'a, T> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            (self.method)(&self.input)
        }
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
    pub fn call(&self, data: T) -> Result<T, error::TransError<T>> {
        self.tx.send(data)?;
        Ok(self.rx.recv()?)
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
/// Gets input from the terminal
/// and returns it as a [String]
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
/// Runs the condition after getting input
/// from the terminal. If nothing goes wrong,
/// return either [Ok] with the contained
/// [boolean] being whether or not the inputted
/// [String] should be returned.
/// However, if something does go wrong,
/// you can return your set error type by
/// having the condition return an [Err]
/// containing the error itsefl.
pub fn input_cond<E>(cond: impl Fn(&String) -> Result<bool, E>) -> Result<String, E> {
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
pub mod error {
    #[derive(Debug)]
    pub enum TransError<T> {
        Send(super::SendError<T>),
        Recv(super::RecvError)
    }
    impl<T> From<super::SendError<T>> for TransError<T> {
        fn from(value: super::SendError<T>) -> Self {
            TransError::Send(value)
        }
    }
    impl<T> From<super::RecvError> for TransError<T> {
        fn from(value: super::RecvError) -> Self {
            TransError::Recv(value)
        }
    }
}