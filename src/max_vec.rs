use std::io::Read;
use std::mem::MaybeUninit;
pub struct MaxVec<T, const N: usize> {
    inner: [MaybeUninit<T>; N],
    len: usize,
}
impl<T, const N: usize> MaxVec<T, N> {
    pub const fn new() -> Self {
        MaxVec {
            inner: [const { MaybeUninit::zeroed() }; N],
            len: 0,
        }
    }
    pub const fn len(&self) -> usize {
        self.len
    }
    pub const fn capacity(&self) -> usize {
        N
    }
    pub const fn remaining(&self) -> usize {
        self.capacity() - self.len()
    }
    pub fn get(&self, index: usize) -> Option<&T> {
        debug_assert!(self.len <= N);
        if index >= self.len {
            return None;
        }
        self.inner
            .get(index)
            .map(|value| unsafe { value.assume_init_ref() })
    }
    pub fn push(&mut self, value: T) -> Result<(), ()> {
        if self.len >= N {
            // Out of space
            return Err(());
        }
        self.inner[self.len] = MaybeUninit::new(value);
        self.len += 1;
        Ok(())
    }
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::mem::transmute(&self.inner[0..self.len]) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { std::mem::transmute(&mut self.inner[0..self.len]) }
    }
}
impl<const N: usize> MaxVec<u8, N> {
    pub fn read_from(&mut self, read: &mut impl Read) -> Result<(), std::io::Error> {
        let inner = unsafe { std::mem::transmute(self.inner.as_mut_slice()) };
        let len = read.read(inner)?;
        self.len = len;
        Ok(())
    }
}
impl<T, const N: usize> std::ops::Index<usize> for MaxVec<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            Some(out) => out,
            None => panic!(
                "Attempted to get element at index {index} but length was {}",
                self.len
            ),
        }
    }
}
impl<T, const N: usize, const L: usize> TryFrom<[T; L]> for MaxVec<T, N> {
    type Error = ();
    fn try_from(value: [T; L]) -> Result<Self, Self::Error> {
        if L > N {
            return Err(());
        }
        let mut out = MaxVec::new();
        for item in value.into_iter() {
            out.push(item).unwrap();
        }
        Ok(out)
    }
}
