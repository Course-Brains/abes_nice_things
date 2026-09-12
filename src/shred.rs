/// This will destroy the data given to it and may leave it in an invalid state. The data should
/// not have its dropper called so look into [std::mem::forget]. Trying to read from the given
/// value after calling this is undefined behavior but writing to it is valid.
///
/// Please note that this will only destroy the memory given to it, so if you give it a Vec<T> then
/// the data held by the Vec will not be destroyed, only the pointer. If you would like to destroy
/// the contents then consider [ShredContents].
pub unsafe fn shred<T>(val: &mut T) {
    let ptr = val as *mut T;
    unsafe {
        ptr.drop_in_place();
        ptr.write_bytes(0, 1);
    }
}

use crate::MaxVec;
use crate::Number;
/// Destroy all data held by a type and leave it in a state which may or may not be valid. This
/// will handle running destructors. Reading the data in any way including destructors is undefined
/// behavior so remember to [std::mem::forget]
pub trait Shred {
    unsafe fn shred(&mut self);
}
/// Destroy all data held by a type but not the type itself so that it can be reused
pub trait ShredContents {
    fn shred_contents(&mut self);
}
impl<T: Shred> ShredContents for Vec<T> {
    fn shred_contents(&mut self) {
        while !self.is_empty() {
            unsafe {
                self.last_mut().unwrap().shred();
            }
            std::mem::forget(self.pop().unwrap());
        }
    }
}
// we also need to shred the unfilled capacity just in case even though it should be zeros
impl<T: Shred, const CAP: usize> ShredContents for MaxVec<T, CAP> {
    fn shred_contents(&mut self) {
        unsafe {
            let slice = self.as_unfilled_slice_mut();
            let ptr = slice.as_mut_ptr();
            ptr.write_bytes(0, slice.len());
        }
        while !self.is_empty() {
            unsafe { self.last_mut().unwrap().shred() }
            std::mem::forget(self.pop().unwrap())
        }
    }
}
impl ShredContents for String {
    fn shred_contents(&mut self) {
        unsafe {
            self.as_mut_vec().shred_contents();
        }
    }
}

macro_rules! basic_shred {
    ($($type:ty)*) => {
        $(
            impl Shred for $type {
                unsafe fn shred(&mut self) {
                    unsafe { shred(self) }
                }
            }
        )*
    }
}
basic_shred!(bool char);
impl<N: Number> Shred for N {
    unsafe fn shred(&mut self) {
        unsafe { shred(self) }
    }
}
impl<T: Shred> Shred for Vec<T> {
    unsafe fn shred(&mut self) {
        self.shred_contents();
        unsafe { (self as *mut Vec<T>).drop_in_place() }
    }
}
impl<T: Shred, const CAP: usize> Shred for MaxVec<T, CAP> {
    unsafe fn shred(&mut self) {
        self.shred_contents();
    }
}
impl Shred for String {
    unsafe fn shred(&mut self) {
        unsafe { self.as_mut_vec().shred() }
    }
}
impl<T: Shred, const N: usize> Shred for [T; N] {
    unsafe fn shred(&mut self) {
        unsafe { shred(self) }
    }
}
impl<T: Shred, U: Shred> Shred for (T, U) {
    unsafe fn shred(&mut self) {
        unsafe {
            self.0.shred();
            self.1.shred();
        }
    }
}
#[test]
fn em_test() {
    let mut data = vec!["foobar".to_string()];
    data.shred_contents();
}
