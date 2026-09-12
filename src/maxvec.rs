use crate::{FromBinary, ToBinary, from_binary};
use std::io::Read;
#[derive(Debug)]
pub struct MaxVec<T, const CAP: usize> {
    len: usize,
    inner: [std::mem::MaybeUninit<T>; CAP],
}
impl<T, const CAP: usize> MaxVec<T, CAP> {
    pub const fn new() -> Self {
        Self {
            len: 0,
            inner: [const { std::mem::MaybeUninit::zeroed() }; CAP],
        }
    }
    /// The used length of the MaxVec. This is not the capacity which is the maximum possible
    /// length.
    pub const fn len(&self) -> usize {
        self.len
    }
    pub const unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }
    pub const fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        Some(unsafe { self.inner[index].assume_init_ref() })
    }
    pub const fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        Some(unsafe { self.inner[index].assume_init_mut() })
    }
    pub fn push(&mut self, value: T) -> Option<usize> {
        if self.len == CAP {
            return None;
        }
        self.inner[self.len] = std::mem::MaybeUninit::new(value);
        self.len += 1;
        Some(self.len)
    }
    pub const fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        self.len -= 1;
        Some(unsafe {
            std::mem::replace(&mut self.inner[self.len], std::mem::MaybeUninit::zeroed())
                .assume_init()
        })
    }
    pub fn as_slice(&self) -> &[T] {
        unsafe { std::mem::transmute(&self.inner[0..self.len]) }
    }
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe { std::mem::transmute(&mut self.inner[0..self.len]) }
    }
    /// Gets the unfilled section of the inner array, reading from this in ANY capacity is unsafe.
    /// However writing to it is completely safe and will cause zero issues so long as doing so
    /// doesn't call any drop functions for the data within which was uninitialized memory. So if
    /// you can guarantee that you will not read any data and nothing that was originally in it
    /// will be dropped and cause issues because of that then go ahead.
    ///
    /// p.s. you can skip the drop call for the data which is here originally, see
    /// [std::mem::forget] for more
    pub unsafe fn as_unfilled_slice_mut(&mut self) -> &mut [T] {
        unsafe { std::mem::transmute(&mut self.inner[self.len..CAP]) }
    }
    /// Will only fail if given insufficient capacity
    pub fn from_array<const N: usize>(array: [T; N]) -> Option<Self> {
        if array.len() > CAP {
            return None;
        }
        let mut out = Self::new();
        for (index, element) in array.into_iter().enumerate() {
            out.inner[index] = std::mem::MaybeUninit::new(element)
        }
        out.len = N;
        Some(out)
    }
    pub fn from_array_exact(array: [T; CAP]) -> Self {
        Self {
            len: array.len(),
            inner: array.map(std::mem::MaybeUninit::new),
        }
    }
    /// Will only fail if given insufficient capacity
    pub fn from_vec(vec: Vec<T>) -> Option<Self> {
        if vec.len() > CAP {
            return None;
        }
        let mut out = Self::new();
        out.len = vec.len();
        for (index, element) in vec.into_iter().enumerate() {
            out.inner[index] = std::mem::MaybeUninit::new(element);
        }
        Some(out)
    }
    pub fn iter<'a>(&'a self) -> std::slice::Iter<'a, T> {
        self.as_slice().iter()
    }
    pub fn iter_mut<'a>(&'a mut self) -> std::slice::IterMut<'a, T> {
        self.as_slice_mut().iter_mut()
    }
    /// Unsafe because len must be accurate as given
    pub const unsafe fn from_raw(len: usize, inner: [std::mem::MaybeUninit<T>; CAP]) -> Self {
        Self { len, inner }
    }
    /// Will replace all values with zeros instead of leaving them as is
    pub fn empty(&mut self) {
        for index in (0..self.len).rev() {
            self.len = index;
            unsafe {
                std::mem::replace(&mut self.inner[index], std::mem::MaybeUninit::zeroed())
                    .assume_init();
            }
        }
    }
    /// Will call destructors if needed, but will NOT remove the data
    pub fn empty_iffy(&mut self) {
        // If no dropping needs to be done, then don't
        if !std::mem::needs_drop::<T>() {
            self.len = 0;
            return;
        }
        for index in (0..self.len).rev() {
            self.len = index;
            unsafe {
                self.inner[index].assume_init_drop();
            }
        }
        debug_assert_eq!(self.len, 0);
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn contains(&self, target: &T) -> bool
    where
        T: PartialEq,
    {
        self.as_slice().contains(target)
    }
    /// Take all data out of the MaxVec and return it if it is full and if it is not then do
    /// nothing. If this returns Some(data) then this will be reset to an empty state.
    pub fn take_if_full(&mut self) -> Option<[T; CAP]> {
        if self.len() == CAP {
            self.len = 0;
            Some(
                std::mem::replace(
                    &mut self.inner,
                    [const { std::mem::MaybeUninit::zeroed() }; CAP],
                )
                .map(|val| unsafe { val.assume_init() }),
            )
        } else {
            None
        }
    }
    pub fn is_full(&self) -> bool {
        self.len() == CAP
    }
    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        Some(&self[self.len() - 1])
    }
    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            return None;
        }
        let len = self.len();
        Some(&mut self[len - 1])
    }
}
impl<const CAP: usize> MaxVec<u8, CAP> {
    /// Essentially like getting the inner array and reading into it then incrementing len as
    /// needed.
    ///
    /// This does just use a read call after with the buffer being the uninitialized section and
    /// increments len.
    pub fn read_from(&mut self, read: &mut impl Read) -> std::io::Result<usize> {
        let unfilled = unsafe { self.as_unfilled_slice_mut() };
        let amount = read.read(unfilled)?;
        self.len += amount;
        Ok(amount)
    }
    pub fn read_from_exact(&mut self, read: &mut impl Read) -> std::io::Result<()> {
        let unfilled = unsafe { self.as_unfilled_slice_mut() };
        read.read_exact(unfilled)?;
        self.len = CAP;
        Ok(())
    }
}
impl<const CAP: usize, T> Drop for MaxVec<T, CAP> {
    fn drop(&mut self) {
        self.empty_iffy();
    }
}
impl<T: Clone, const CAP: usize> Clone for MaxVec<T, CAP> {
    fn clone(&self) -> Self {
        let mut out = Self::new();
        out.len = self.len;
        for (index, element) in self.as_slice().iter().enumerate() {
            out.inner[index] = std::mem::MaybeUninit::new(element.clone());
        }
        out
    }
}
impl<T, const CAP: usize> std::ops::Index<usize> for MaxVec<T, CAP> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!(
                "Attempted to access an uninitialized index: {index} when initialized up to {}",
                self.len
            );
        }
        unsafe { self.inner[index].assume_init_ref() }
    }
}
impl<T, const CAP: usize> std::ops::IndexMut<usize> for MaxVec<T, CAP> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!(
                "Attempted to access an uninitialized index: {index} when initialized up to {}",
                self.len
            )
        }
        unsafe { self.inner[index].assume_init_mut() }
    }
}
impl<T: PartialEq, const CAP: usize> PartialEq for MaxVec<T, CAP> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
    fn ne(&self, other: &Self) -> bool {
        self.as_slice() != other.as_slice()
    }
}
impl<T: Eq, const CAP: usize> Eq for MaxVec<T, CAP> {}
impl<T: FromBinary, const CAP: usize> FromBinary for MaxVec<T, CAP> {
    fn from_binary(binary: &mut dyn std::io::Read) -> from_binary::Result<Self> {
        match MaxVec::from_vec(<Vec<T>>::from_binary(binary)?) {
            Some(max_vec) => Ok(max_vec),
            None => from_binary::err(
                std::io::ErrorKind::InvalidData,
                "Failed to get MaxVec from \
            binary due to illegal size",
            ),
        }
    }
}
impl<T: ToBinary, const CAP: usize> ToBinary for MaxVec<T, CAP> {
    fn to_binary(&self, binary: &mut dyn std::io::Write) -> from_binary::Result<()> {
        self.as_slice().to_binary(binary)
    }
}
impl<const CAP: usize> std::io::Write for MaxVec<u8, CAP> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut written = 0;
        for byte in buf.iter() {
            if self.push(*byte).is_none() {
                break;
            }
            written += 1;
        }
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl<T, const CAP: usize> AsRef<[T]> for MaxVec<T, CAP> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
impl<T, const CAP: usize> AsMut<[T]> for MaxVec<T, CAP> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_slice_mut()
    }
}
