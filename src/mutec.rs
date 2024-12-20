use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::Thread;
use std::collections::VecDeque;
/// Lower level version of [Mutex](std::sync::Mutex)
/// which is just an [AtomicBool] and your value.
#[derive(Debug)]
pub struct Atomex<T> {
    atomic: AtomicBool,
    data: UnsafeCell<T>
}
impl<T> Atomex<T> {
    pub fn new(data: T) -> Self {
        Atomex {
            atomic: AtomicBool::new(false),
            data: UnsafeCell::new(data),
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
                unsafe { Ok(self.data.get().as_mut().unwrap()) }
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
    inner: Vec<(Atomex<UnsafeCell<T>>, Atomex<VecDeque<Thread>>)>,
}
// Block for methods relating to the lock/unlock itself
impl<T> Mutec<T> {
    pub fn check_lock(&self, index: usize) -> bool {
        self.inner[index].0.check_lock()
    }
    pub fn lock(&self, index: usize) -> MutecGuard<T> {
        loop {
            match self.inner[index].0.try_lock() {
                Ok(data) => {
                    // Lock is free
                    return MutecGuard {
                        inner: unsafe { data.get().as_mut() }.unwrap(),
                        parent: self,
                        index
                    }
                }
                Err(_) => {
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
    pub fn try_lock(&self, index: usize) -> Result<MutecGuard<T>, ()> {
        match self.inner[index].0.try_lock() {
            Ok(data) => {
                return Ok(MutecGuard {
                    inner: unsafe { data.get().as_mut() }.unwrap(),
                    parent: self,
                    index
                })
            }
            Err(_) => {
                return Err(())
            }
        }
    }
    unsafe fn unlock(&self, index: usize) {
        self.inner[index].0.unlock();
        if let Ok(data) = self.inner[index].1.try_lock() {
            if let Some(thread) = data.pop_front() {
                thread.unpark()
            }
        }
    }
}
// Block for methods relating to it as a Vec wrapper
impl<T> Mutec<T> {
    pub const fn new() -> Mutec<T> {
        Mutec {
            inner: Vec::new()
        }
    }
    pub fn push(&mut self, value: T) {
        self.inner.push((
            Atomex::new(
                UnsafeCell::new(value)
            ),
            Atomex::new(
                VecDeque::new()
            ),
        ))
    }
    pub fn len(&self) -> usize {
        self.inner.len()
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
    /// This creates a [Vec] of the [MutecGuards](MutecGuard)
    /// which is situationally more useful than just [iterating](Iter)
    /// throught it. If you just [iterate](Iter) through it, it will
    /// obtain the [lock](Mutec::lock) of a value, give it to you, then when you call
    /// [next](Iterator::next) again, it will get the [lock](Mutec::lock) of the next one.
    /// However, you might need it to obtain all the [locks](Mutec::lock), then
    /// have it give you the values. In which case, iterating through what this
    /// gives would be usefull:
    /// ```
    /// # use abes_nice_things::mutec::Mutec;
    /// for mutec_guard in Mutec::from([1,2,3,4]).to_vec().iter() {
    ///     // Whatever it is you want to do with it
    /// }
    /// ```
    pub fn to_vec(&self) -> Vec<MutecGuard<T>> {
        let mut vec: Vec<MutecGuard<T>> = Vec::with_capacity(self.len());
        for index in 0..self.len() {
            vec.push(self.lock(index))
        }
        vec
    }
    pub fn iter(&self) -> Iter<T> {
        Iter {
            parent: self,
            index: 0,
            back_index: self.inner.len()
        }
    }
    pub fn async_iter(&self) -> AsyncIter<T> {
        let mut progress: Vec<bool> = Vec::with_capacity(self.len());
        for _ in 0..self.len() {
            progress.push(false);
        }
        AsyncIter {
            parent: self,
            progress
        }
    }
}
// General trait implementations
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
unsafe impl<T> Sync for Mutec<T> {}
unsafe impl<T> Send for Mutec<T> {}
/// [Iter](Iterator) struct for [Mutec]
pub struct Iter<'a, T> {
    parent: &'a Mutec<T>,
    index: usize,
    back_index: usize,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = MutecGuard<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.back_index {
            return None
        }
        self.index += 1;
        Some(self.parent.lock(self.index-1))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}
impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.back_index <= self.index {
            return None
        }
        self.back_index -= 1;
        Some(self.parent.lock(self.back_index))
    }
}
impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.back_index - self.index
    }
}
impl<'a, T> std::iter::FusedIterator for Iter<'a, T> {}
pub struct AsyncIter<'a, T> {
    parent: &'a Mutec<T>,
    progress: Vec<bool>,
}
impl<'a, T> Iterator for AsyncIter<'a, T> {
    type Item = MutecGuard<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut done: bool = false;
        while !done {
            done = true;
            for (index, progress) in self.progress.iter_mut().enumerate() {
                if *progress {
                    continue;
                }
                done = false;
                if let Ok(guard) = self.parent.try_lock(index) {
                    *progress = true;
                    return Some(guard)
                }
            }
        }
        return None
    }
}
#[derive(Debug)]
pub struct MutecGuard<'a, T> {
    inner: &'a mut T,
    parent: &'a Mutec<T>,
    index: usize,
}
impl<'a, T> Drop for MutecGuard<'a, T> {
    fn drop(&mut self) {
        unsafe { self.parent.unlock(self.index) }
    }
}
use std::ops::{Deref, DerefMut};
impl<'a, T> Deref for MutecGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
impl<'a, T> DerefMut for MutecGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
impl<'a, T: PartialEq> PartialEq for MutecGuard<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}
impl<'a, T: PartialEq> PartialEq<T> for MutecGuard<'a, T> {
   fn eq(&self, other: &T) -> bool {
       self.deref().eq(other)
   }
}
#[cfg(test)]
mod tests {
    mod atomic_lock {
        use super::super::*;
        #[test]
        fn basic_get_value() {
            let atomex: Atomex<usize> = Atomex::new(5);
            assert_eq!(
                *atomex.try_lock().expect("failed to get available lock"),
                5,
                "Value was changed in locking"
            )
        }
        #[test]
        fn basic_all() {
            let atomex: Atomex<usize> = Atomex::new(7);
            assert!(!atomex.check_lock(),"Atomex was locked on creation");
            atomex.try_lock().expect("failed to acquire lock");
            assert!(atomex.check_lock(), "Atomex was unlocked after lock was acquired");
            atomex.try_lock().expect_err("Lock was gained while locked");
            unsafe { atomex.unlock() }
            assert!(!atomex.check_lock(), "Atomex was locked after unlocking");
        }
    }
    mod mutec {
        use super::super::*;
        #[test]
        fn basic_lock() {
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
            };
        }
        #[test]
        fn basic_int_mut() {
            let mutec: Mutec<&str> = Mutec::from(["the"]);
            *mutec.lock(0) = "pencil";
            assert_eq!(*mutec.lock(0), "pencil", "Value was not correct");
        }
        #[test]
        fn basic_thread_lock() {
            let mutec: Mutec<&str> = Mutec::from(["is"]);
            let mut guard = mutec.lock(0);
            std::thread::scope(|s| {
                s.spawn(|| {
                    let mut mutec_guard = mutec.lock(0);
                    assert_eq!(*mutec_guard, "hungry", "pain");
                    *mutec_guard = "today";
                });
                assert_eq!(*guard, "is", "fuck");
                *guard = "hungry";
                drop(guard)
            });
            assert_eq!(*mutec.lock(0), "today", "BLYAT");
        }
        mod iter {
            use super::super::super::*;
            #[test]
            fn next_basic() {
                let list: [&str; 11] = ["5","6","7","8","9","10","11","12","13","15","16"];
                let mutec: Mutec<&str> = Mutec::from(list);
                let mut mutec_iter = mutec.iter();
                for (index, item) in list.iter().enumerate() {
                    assert_eq!(
                        *item,
                        *mutec_iter.next().expect("mutec_iter was too short"),
                        "Iterators were not equivalent at index: {index}",
                    )
                }
            }
            #[test]
            fn next_back_basic() {
                let list: [&str; 11] = ["5","6","7","8","9","10","11","12","13","15","16"];
                let mutec: Mutec<&str> = Mutec::from(list);
                let mut mutec_iter = mutec.iter();
                for (index, item) in list.iter().rev().enumerate() {
                    assert_eq!(
                        *item,
                        *mutec_iter.next_back().expect("mutec_iter was too short"),
                        "Iterators were not equivalent at index: {index}"
                    )
                }
            }
            #[test]
            fn next_over() {
                let mutec: Mutec<&str> = Mutec::from(["5","6","7"]);
                let mut mutec_iter = mutec.iter();
                mutec_iter.next();
                mutec_iter.next();
                mutec_iter.next();
                assert!(mutec_iter.next().is_none(), "mutec iter had an extra value");
                assert!(mutec_iter.next().is_none(), "mutec iter had an extra value");
                assert!(mutec_iter.next().is_none(), "mutec iter had an extra value");
            }
            #[test]
            fn next_back_over() {
                let mutec: Mutec<&str> = Mutec::from(["5","6","7"]);
                let mut mutec_iter = mutec.iter();
                mutec_iter.next_back();
                mutec_iter.next_back();
                mutec_iter.next_back();
                assert!(mutec_iter.next_back().is_none(), "mutec iter had an extra value");
                assert!(mutec_iter.next_back().is_none(), "mutec iter had an extra value");
                assert!(mutec_iter.next_back().is_none(), "mutec iter had an extra value");
            }
        }
        mod async_iter {
            use super::super::super::Mutec;
            #[test]
            fn next_basic() {
                let source: &[usize] = &[5, 2, 7, 42, 79];
                let mutec: Mutec<usize> = Mutec::from(source);
                let mut check: Vec<usize> = Vec::with_capacity(source.len());
                for item in mutec.async_iter() {
                    check.push(*item);
                }
                assert_eq!(source, check);
            }
            #[test]
            fn next_over() {
                let source: &[usize] = &[5, 9, 23];
                let mutec: Mutec<usize> = Mutec::from(source);
                let mut iter = mutec.async_iter();
                iter.next();
                iter.next();
                iter.next();
                assert_eq!(iter.next(), None);
                assert_eq!(iter.next(), None);
            }
            /*#[test]
            fn order() {
                // 1: 1(hold)

                // 2: 2(hold)
                // swap
                // 1: 3,4
                // 1: drop 1

                // 2: 3,4,1
                // 2: drop 2
                // 2 done

                // 1: 2
                // 1 done
                let source: &[usize] = &[8, 2, 6, 1];
                let mutec: Mutec<usize> = Mutec::from(source);
                let mut iter1 = mutec.async_iter();
                let mut iter2 = mutec.async_iter();
                
                // 1: 1(hold)
                let val1 = iter1.next().unwrap();
                assert_eq!(*val1, source[0]);
                // 2: 2(hold)
                let val2 = iter2.next().unwrap();
                assert_eq!(*val2, source[1]);
                // 1: 3,4 & drop 1
                assert_eq!(*iter1.next().unwrap(), source[2]);
                assert_eq!(*iter1.next().unwrap(), source[3]);
                // drop(val1);
                // 2: 3,4,1 & drop 2
                assert_eq!(*iter2.next().unwrap(), source[2]);
                assert_eq!(*iter2.next().unwrap(), source[3]);
                assert_eq!(*iter2.next().unwrap(), source[0]);
                drop(val2);
                // 2: done
                // 1: 2
                assert_eq!(*iter1.next().unwrap(), source[1]);
                // 1: done
            }*/
        }
    }
}