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
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::Thread;
use std::collections::VecDeque;
/// Lower level version of [Mutex](std::sync::Mutex)
/// which is just an [AtomicBool] and your value.
#[derive(Debug)]
pub struct AtomicLock<T> {
    atomic: AtomicBool,
    data: T
}
impl<T> AtomicLock<T> {
    pub fn new(data: T) -> Self {
        AtomicLock {
            atomic: AtomicBool::new(false),
            data
        }
    }
    pub fn try_lock(&self) -> Result<&mut T, ()> {
        match self.atomic.compare_exchange(
            false,
            true,
            Ordering::SeqCst,
            Ordering::SeqCst
        ) {
            Ok(_) => {
                unsafe { Ok(&mut *(&self.data as *const T as *mut T) ) }
            }
            Err(_) => {
                return Err(())
            }
        }
    }
    pub unsafe fn unlock(&self) {
        self.atomic.store(false, Ordering::SeqCst)
    }
    pub fn check_lock(&self) -> bool {
        self.atomic.load(Ordering::SeqCst)
    }
}
#[derive(Debug)]
pub struct Mutec<T> {
    inner: Vec<(AtomicLock<UnsafeCell<T>>, AtomicLock<VecDeque<Thread>>)>,
}
impl<T> Mutec<T> {
    pub const fn new() -> Mutec<T> {
        Mutec {
            inner: Vec::new()
        }
    }
    pub fn push(&mut self, value: T) {
        self.inner.push((
            AtomicLock::new(
                UnsafeCell::new(value)
            ),
            AtomicLock::new(
                VecDeque::new()
            ),
        ))
    }
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), std::collections::TryReserveError> {
        self.inner.try_reserve_exact(additional)
    }
    pub fn extend_from_slice(&mut self, slice: &[T]) where T: Clone {
        if let Err(error) = self.try_reserve_exact(slice.len()) {
            panic!("Error reserving required space: {error}");
        }
        for item in slice.into_iter() {
            self.push(item.clone());
        }
    }
    pub fn check_lock(&self, index: usize) -> bool {
        self.inner[index].0.check_lock()
    }
    pub fn lock(&self, index: usize) -> MutecLock<T> {
        loop {
            match self.inner[index].0.try_lock() {
                Ok(data) => {
                    // Lock is free
                    return MutecLock {
                        inner: data.get(),
                        parent: self as *const Mutec<T>,
                        index
                    }
                }
                Err(_) => {
                    println!("Lock was not free");
                    // Lock is not free
                    if let Ok(data) = self.inner[index].1.try_lock() {
                        // Can modify list
                        data.push_back(std::thread::current());
                        unsafe { self.inner[index].1.unlock() }
                        std::thread::park();
                    }
                }
            }
        }
    }
    pub fn try_lock(&self, index: usize) -> Result<MutecLock<T>, ()> {
        match self.inner[index].0.try_lock() {
            Ok(data) => {
                return Ok(MutecLock {
                    inner: data.get(),
                    parent: self as *const Mutec<T>,
                    index
                })
            }
            Err(_) => {
                return Err(())
            }
        }
    }
    pub unsafe fn unlock(&self, index: usize) {
        self.inner[index].0.unlock();
        if let Ok(data) = self.inner[index].1.try_lock() {
            if let Some(thread) = data.pop_front() {
                thread.unpark()
            }
        }
    }
}
impl<T: Clone> From<&[T]> for Mutec<T> {
    fn from(slice: &[T]) -> Self {
        let mut mutec: Mutec<T> = Mutec::new();
        mutec.extend_from_slice(slice);
        mutec
    }
}
impl<T, const N: usize> From<[T; N]> for Mutec<T> {
    fn from(value: [T; N]) -> Self {
        let mut mutec: Mutec<T> = Mutec::new();
        mutec.extend(value);
        mutec
    }
}
impl<T> From<Vec<T>> for Mutec<T> {
    fn from(value: Vec<T>) -> Self {
        let mut mutec: Mutec<T> = Mutec::new();
        mutec.extend(value);
        mutec
    }
}
impl<T> FromIterator<T> for Mutec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut mutec: Mutec<T> = Mutec::new();
        for item in iter {
            mutec.push(item);
        }
        mutec
    }
}
impl<T> Extend<T> for Mutec<T> {
    fn extend<A: IntoIterator<Item = T>>(&mut self, iter: A) {
        // Not sure if reserving here is faster bc
        // It would mean having to go through the iter
        // twice...
        // TODO: find out
        for item in iter.into_iter() {
            self.push(item)
        }
    }
}
pub struct MutecLock<T> {
    inner: *mut T,
    parent: *const Mutec<T>,
    index: usize,
}
impl<T> Drop for MutecLock<T> {
    fn drop(&mut self) {
        unsafe { self.parent.as_ref().unwrap().unlock(self.index) }
    }
}
use std::ops::{Deref, DerefMut};
impl<T> Deref for MutecLock<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }.unwrap()
    }
}
impl<T> DerefMut for MutecLock<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }.unwrap()
    }
}
#[cfg(test)]
mod tests {
    mod nvec {}
    mod atomic_lock {}
    mod mutec {
        use super::super::*;
        #[test]
        fn basic_lock_usize() {
            println!("Creating Mutec");
            let mutec: Mutec<usize> = Mutec::from([1]);
            println!("Locking");
            let lock = mutec.lock(0);
            println!("Checking value");
            assert_eq!(*lock, 1, "Read failed, was: {}, expected 1", *lock);
        }
        #[test]
        fn check_lock() {
            let mutec: Mutec<usize> = Mutec::from([1]);
            assert!(!mutec.check_lock(0), "Initial false positive");
            let _guard = mutec.lock(0);
            assert!(mutec.check_lock(0), "false negative");
            drop(_guard);
            assert!(!mutec.check_lock(0), "Post false positive");
        }
        #[test]
        fn basic_unique_lock() {
            println!("Creating Mutec");
            let mutec: Mutec<usize> = Mutec::from([1]);
            println!("Locking");
            let _guard = mutec.lock(0);
            println!("Attempting illegal lock");
            if let Ok(_) = mutec.try_lock(0) {
                panic!("Aquired lock while lock already exists")
            }
        }
        #[test]
        fn empty_creation() {
            let _mutec: Mutec<usize> = Mutec::new();
        }
    }
}