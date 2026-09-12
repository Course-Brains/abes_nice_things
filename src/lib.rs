//! A collection of types, functions, traits, and macros which
//! I found to be helpful and improve my experience while programming and didn't feel like
//! programming again

mod from_binary;
pub use from_binary::{FromBinary, ToBinary};
mod input;
pub use input::{Input, input};
pub mod split;
pub use split::Split;
pub mod numbers;
pub use numbers::*;
pub mod progress_bar;
pub use progress_bar::ProgressBar;
pub mod style;
pub use style::{Color, Style};
mod log;
pub use log::{logln, set_log_path};
pub mod aes;
mod maxvec;
#[cfg(unix)]
pub mod random;
pub use maxvec::*;
pub mod shred;

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
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("[DEBUG] {}", format_args!($($arg)*));
    }
}
/// A macro which will only run code
/// when the crate is not compiled
/// with '--release'
///
/// Specifically this places whatever tokens given to it in a block which will only be included in
/// debug. For example,
///
/// ```
/// # use abes_nice_things::debug;
/// # fn main() {
/// println!("Works in both release and debug");
///
/// debug!(println!("This is debug only"));
///
/// debug!({
///     println!("This is also debug only");
/// });
/// # }
/// ```
#[macro_export]
macro_rules! debug {
    ($($token:tt)*) => {
        #[cfg(debug_assertions)]
        {$($token)*}
    };
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
// TODO: stop using this and get rid of it
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
/// A macro which will pass through whatever tokens you give it on debug, but will fail to compile
/// on release.
///
/// Because this passes any tokens given to it, you can put debugging placeholder values in it and
/// you will not be able to compile for release until you replace them.
///
/// ```
/// # use abes_nice_things::require_debug;
/// # fn main() {
/// let mut value = require_debug!(7);
///
/// println!("{value}");
/// # }
/// ```
#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! require_debug {
    ($($tokens:tt)*) => {
        compile_error!("Attempted to compile debug only code in release");
    };
}
#[cfg(debug_assertions)]
#[macro_export]
macro_rules! require_debug {
    ($($token:tt)*) => {
        $($token)*
    }
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
        {$($tokens)*}
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
        {$($tokens)*}
    };
}

/// Thread safe version.
///
/// Use with Arc<T> instead of just T to get a reference instead of cloning. Does put your data on
/// the heap though so which is more expensive? A Clone every read or a heap allocation every
/// write?
///
/// Hint hint: If something implements Copy then it has very very cheap Clones because it will just
/// Copy it.
#[derive(Debug)]
pub struct ResetLazyLock<T: Clone, F: Fn() -> T = fn() -> T> {
    data: std::sync::RwLock<Option<T>>,
    closure: F,
}
impl<T: Clone, F: Fn() -> T> ResetLazyLock<T, F> {
    pub const fn new(closure: F) -> Self {
        Self {
            data: std::sync::RwLock::new(None),
            closure,
        }
    }
    pub fn get(&self) -> Result<T, Box<dyn std::error::Error + '_>> {
        loop {
            let read = self.data.read()?;
            if read.is_none() {
                std::mem::drop(read);
                self.recalc()?;
            } else {
                // Safety: We know that the value has not been dropped by a different thread
                // because we maintained a shared lock of the data through checking if it is None
                // until now. TOCTOU errors BEGONE!
                break Ok(read.clone().unwrap());
            }
        }
    }
    pub fn recalc(
        &self,
    ) -> Result<(), std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, Option<T>>>> {
        let mut lock = self.data.write()?;
        *lock = Some((self.closure)());
        Ok(())
    }
    pub fn drop(
        &self,
    ) -> Result<(), std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, Option<T>>>> {
        *self.data.write()? = None;
        Ok(())
    }
    pub fn take(
        &self,
    ) -> Result<T, std::sync::PoisonError<std::sync::RwLockWriteGuard<'_, Option<T>>>> {
        let mut lock = self.data.write()?;
        // Can't match because it is in a guard
        if lock.is_none() {
            Ok((self.closure)())
        } else {
            // Safety: Again, safe because we maintained the lock in between the check and now
            Ok(lock.take().unwrap())
        }
    }
    pub fn is_init(
        &self,
    ) -> Result<bool, std::sync::PoisonError<std::sync::RwLockReadGuard<'_, Option<T>>>> {
        Ok(self.data.read()?.is_some())
    }
    /// True if triggered recalc
    pub fn recalc_if_uninit(&self) -> Result<bool, Box<dyn std::error::Error + '_>> {
        // Cheaply checking if we don't need to get a write lock
        if self.is_init()? {
            return Ok(false);
        }
        let mut lock = self.data.write()?;
        // Have to check if it got initialized between the release of the read and getting the
        // write
        if lock.is_some() {
            return Ok(false);
        }
        *lock = Some((self.closure)());
        Ok(true)
    }
}
#[derive(Clone, Debug)]
pub struct ResetLazyCell<T: Clone, F: Fn() -> T = fn() -> T> {
    data: std::cell::RefCell<Option<T>>,
    closure: F,
}
impl<T: Clone, F: Fn() -> T> ResetLazyCell<T, F> {
    pub const fn new(closure: F) -> Self {
        Self {
            data: std::cell::RefCell::new(None),
            closure,
        }
    }
    pub fn get(&self) -> T {
        let read = self.data.borrow();
        if read.is_none() {
            std::mem::drop(read);
            self.recalc();
            self.data.borrow().clone().unwrap()
        } else {
            read.clone().unwrap()
        }
    }
    pub fn recalc(&self) {
        let mut lock = self.data.borrow_mut();
        *lock = Some((self.closure)())
    }
    pub fn drop(&self) {
        *self.data.borrow_mut() = None;
    }
    pub fn take(&self) -> T {
        let mut lock = self.data.borrow_mut();
        if lock.is_none() {
            return (self.closure)();
        }
        lock.take().unwrap()
    }
    pub fn is_init(&self) -> bool {
        self.data.borrow().is_some()
    }
    /// True if caused recalc
    pub fn recalc_if_uninit(&self) -> bool {
        if self.is_init() {
            return false;
        }
        let mut write = self.data.borrow_mut();
        if write.is_some() {
            return false;
        }
        *write = Some((self.closure)());
        true
    }
}
pub struct OnDrop<F: Fn() = fn()>(F);
impl<F: Fn()> OnDrop<F> {
    pub fn new(on_drop: F) -> OnDrop<F> {
        OnDrop(on_drop)
    }
}
impl<F: Fn()> Drop for OnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}

/// Compact eight bools into one u8.
///
/// The inverse operation of [expand]
pub fn compact(values: [bool; 8]) -> u8 {
    let mut out = 0;
    for (place, value) in values.into_iter().enumerate() {
        if value {
            out |= 0b1 << place
        }
    }
    out
}
/// Expand one u8 into eight bools.
///
/// The inverse operation of [compact]
pub fn expand(value: u8) -> [bool; 8] {
    let mut out = [false; 8];
    for place in 0..8 {
        out[place] = (value & (0b1 << place)) != 0
    }
    out
}

#[cfg(test)]
mod test {
    use super::*;
    mod max_vec {
        use super::*;
        use std::io::Write;
        #[test]
        #[should_panic = "bip bap"]
        fn mid_drop_panic() {
            struct W(std::mem::ManuallyDrop<String>);
            impl Drop for W {
                fn drop(&mut self) {
                    println!("Dropping: {}", &*self.0);
                    let should_panic = &*self.0 == "b";
                    unsafe {
                        std::mem::ManuallyDrop::drop(&mut self.0);
                    }
                    if should_panic {
                        panic!("bip bap");
                    }
                }
            }
            fn w(string: &str) -> W {
                W(std::mem::ManuallyDrop::new(string.to_string()))
            }
            println!("Creating max_vec");
            let _max_vec: MaxVec<W, 3> = MaxVec::from_array_exact([w("a"), w("b"), w("c")]);
        }
        #[test]
        fn push_get() {
            let mut max_vec: MaxVec<String, 5> = MaxVec::new();
            assert_eq!(max_vec.len(), 0);
            max_vec.push("a".to_string());
            max_vec.push("b".to_string());
            max_vec.push("c".to_string());
            max_vec.push("d".to_string());
            assert_eq!(max_vec.len(), 4);
            assert_eq!(max_vec.get(0), Some(&"a".to_string()));
            assert_eq!(max_vec.get(1), Some(&"b".to_string()));
            assert_eq!(max_vec.get(2), Some(&"c".to_string()));
            assert_eq!(max_vec.get(3), Some(&"d".to_string()));
            assert_eq!(max_vec.get(4), None);
            assert_eq!(max_vec.get(5), None);
        }
        #[test]
        fn index() {
            let max_vec = MaxVec::from_array_exact([0, 1, 2]);
            assert_eq!(max_vec[0], 0);
            assert_eq!(max_vec[1], 1);
            assert_eq!(max_vec[2], 2);
        }
        #[test]
        fn iter() {
            let source = [0, 1, 2, 3];
            assert_eq!(
                Vec::from(source),
                MaxVec::from_array_exact(source)
                    .iter()
                    .map(|element| *element)
                    .collect::<Vec<i32>>()
            );
        }
        #[test]
        fn slice() {
            let source = [0, 1, 2, 3];
            assert_eq!(&source, MaxVec::from_array_exact(source).as_slice())
        }
        #[test]
        fn write() {
            let source = [0, 1, 2, 3];
            let mut max_vec: MaxVec<u8, 4> = MaxVec::new();
            max_vec.write_all(&source).unwrap();
            assert_eq!(&source, max_vec.as_slice());
        }
        #[test]
        fn read_from() {
            let source = [0, 1, 2, 3];
            let mut max_vec: MaxVec<u8, 4> = MaxVec::new();
            assert_eq!(
                max_vec.read_from(&mut source.as_slice()).unwrap(),
                4,
                "I am bad at code analysis"
            );
            assert_eq!(&source, max_vec.as_slice());
        }
        // Reading into the MaxVec but there is more data than capacity
        #[test]
        fn read_from_ooc_truncate() {
            let source = [0, 1, 2, 3];
            let mut max_vec: MaxVec<u8, 3> = MaxVec::new();
            assert_eq!(max_vec.read_from(&mut source.as_slice()).unwrap(), 3);
            assert_eq!(max_vec.as_slice(), &[0, 1, 2]);
        }
        // Reading into the MaxVec but there is less data than capacity
        #[test]
        fn read_from_insufficient_data() {
            let source = [0, 1, 2, 3];
            let mut max_vec: MaxVec<u8, 10> = MaxVec::new();
            assert_eq!(max_vec.read_from(&mut source.as_slice()).unwrap(), 4);
            assert_eq!(max_vec.as_slice(), &source);
        }
        // Reading into the MaxVec but there is less data than capacity, then new data from a different
        // source is used to fill the rest
        #[test]
        fn read_from_insufficient_into_alt_source() {
            let source1 = [0, 1, 2, 3];
            let source2 = [9, 8, 7, 6];
            let mut max_vec: MaxVec<u8, 8> = MaxVec::new();
            assert_eq!(max_vec.read_from(&mut source1.as_slice()).unwrap(), 4);
            assert_eq!(max_vec.read_from(&mut source2.as_slice()).unwrap(), 4);
            assert_eq!(max_vec.as_slice(), &[0, 1, 2, 3, 9, 8, 7, 6]);
        }
        #[test]
        fn empty() {
            let mut max_vec = MaxVec::from_array_exact([0, 1, 2, 3]);
            assert_eq!(max_vec.len(), 4);
            max_vec.empty();
            assert_eq!(max_vec.len(), 0);
        }
        #[test]
        fn empty_iffy() {
            let mut max_vec = MaxVec::from_array_exact([0, 1, 2, 3]);
            assert_eq!(max_vec.len(), 4);
            max_vec.empty_iffy();
            assert_eq!(max_vec.len(), 0);
        }
    }
    #[test]
    fn compact_expand_parity() {
        for test in 0..=u8::MAX {
            assert_eq!(test, compact(expand(test)))
        }
    }
}
