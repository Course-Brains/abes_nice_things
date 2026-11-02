use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::panic::RefUnwindSafe;
use std::panic::UnwindSafe;
use std::sync::Once;
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

union Data<T, F> {
    data: ManuallyDrop<T>,
    function: ManuallyDrop<F>,
}

pub struct ArgLazyLock<A, T, F: FnOnce(A) -> T = fn(A) -> T> {
    once: Once,
    data: UnsafeCell<Data<T, F>>,
    _phantom: std::marker::PhantomData<A>,
}
impl<A, T, F: FnOnce(A) -> T> ArgLazyLock<A, T, F> {
    pub const fn new(function: F) -> ArgLazyLock<A, T, F> {
        // SAFETY: We are putting a function in the data and the Once is incomplete
        Self {
            once: Once::new(),
            data: UnsafeCell::new(Data {
                function: ManuallyDrop::new(function),
            }),
            _phantom: std::marker::PhantomData,
        }
    }
    pub fn init(&self, arg: A) {
        self.once.call_once(|| {
            // SAFETY: Before the once is used, the data is the function so it will be the function
            // Additionally, because the once can only be used once, we know that the function will
            // only be used once. As such we know that no other threads are interacting with the
            // data being held, so we can safely treat it as locked.
            // Also, if this gets run at the same time as another thread then the second thread
            // gets blocked by Once so we don't run into issues there either.
            let data = unsafe { &mut *self.data.get() };
            let function = unsafe { ManuallyDrop::take(&mut data.function) };
            data.data = ManuallyDrop::new((function)(arg));

            // If we reach this point then the function did not panic, meaning that there is a
            // valid value in data. If it did panic then data holds an invalid closure which will
            // never be read again because the Once has been poisoned.
        });
    }
    pub fn wait(&self) {
        self.once.wait();
    }
    pub unsafe fn get_unchecked(&self) -> &T {
        unsafe { (&self.data.get().as_ref_unchecked().data).deref() }
    }
    pub fn get(&self, arg: A) -> &T {
        self.init(arg);
        unsafe { self.get_unchecked() }
    }
}
impl<A, T, F: FnOnce(A) -> T> std::ops::Drop for ArgLazyLock<A, T, F> {
    fn drop(&mut self) {
        // SAFETY: Due to using a Once, data will be in one of the following states:
        // Incomplete = need to drop function
        // Complete = need to drop data
        // Poisoned = neither are safe to drop
        if self.once.is_completed() {
            unsafe { ManuallyDrop::drop(&mut self.data.get_mut().data) };
            return;
        }
        // If we reach this point then the Once is either Incomplete or Poisoned, in which case
        // using call_once_force will always call the given function and is the only way to get the
        // state.
        self.once.call_once_force(|state| {
            // If it is poisoned then doing nothing is the only safe option, but if it isn't then
            // we need to drop the function
            if !state.is_poisoned() {
                unsafe { ManuallyDrop::drop(&mut self.data.get_mut().function) };
            }
        });
        // Technically this unpoisons the Once which would result in reading the union as data even
        // though there isn't any but it is ok becuse this is during the drop so it won't be used
        // after this
    }
}

// The function does not need Sync because it is only ever used by one thread
unsafe impl<A, T: Send + Sync, F: FnOnce(A) -> T + Send + Sync> Sync for ArgLazyLock<A, T, F> {}
// Due to Once poisoning this is unwind safe as well
impl<A, T: UnwindSafe + RefUnwindSafe, F: FnOnce(A) -> T + UnwindSafe> RefUnwindSafe
    for ArgLazyLock<A, T, F>
{
}
impl<A, T: UnwindSafe, F: FnOnce(A) -> T + UnwindSafe> UnwindSafe for ArgLazyLock<A, T, F> {}

#[cfg(test)]
mod tests {
    use super::*;
    mod arg_lazy_lock {
        use super::*;
        #[test]
        fn single_thread() {
            let lock = ArgLazyLock::new(|arg| arg * 2);
            lock.get(5);
            assert_eq!(*lock.get(0), 10);
        }
        #[test]
        fn single_thread_static() {
            static LOCK: ArgLazyLock<usize, String> = ArgLazyLock::new(|arg| arg.to_string());
            LOCK.get(10);
            assert_eq!(LOCK.get(5).as_str(), "10");
        }
    }
}
