use crate::Binary;
use std::sync::atomic::*;
/// A trait which is implemented by all numbers
/// (All signed integers, all unsigned integers,
/// and all floats){except not the nightly ones,
/// fuck that shit}
///
/// Pretty much, you can use it as a generic
/// constraint and get access to all the traits
/// numbers implement(it's a lot).
///
/// If you do not know how to use generic
/// constraints, I recommend reading the Rust
/// book.
///
/// If you need more specific constraints,
/// use [Integer], [UnsignedInteger],
/// [SignedInteger], or [Float]
pub unsafe trait Number
where
    Self: Sized
        + std::ops::Add<Output = Self>
        + for<'a> std::ops::Add<&'a Self>
        + std::ops::AddAssign
        + for<'a> std::ops::AddAssign<&'a Self>
        + std::ops::Sub<Output = Self>
        + for<'a> std::ops::Sub<&'a Self>
        + std::ops::SubAssign
        + for<'a> std::ops::SubAssign<&'a Self>
        + std::ops::Mul<Output = Self>
        + for<'a> std::ops::Mul<&'a Self>
        + std::ops::MulAssign
        + for<'a> std::ops::MulAssign<&'a Self>
        + std::ops::Div<Output = Self>
        + for<'a> std::ops::Div<&'a Self>
        + std::ops::DivAssign
        + for<'a> std::ops::DivAssign<&'a Self>
        + std::ops::Rem<Output = Self>
        + for<'a> std::ops::Rem<&'a Self>
        + std::ops::RemAssign
        + for<'a> std::ops::RemAssign<&'a Self>
        + std::iter::Product
        + for<'a> std::iter::Product<&'a Self>
        + std::iter::Sum
        + for<'a> std::iter::Sum<&'a Self>
        + PartialEq
        + PartialOrd
        + Clone
        + Copy
        + std::fmt::Debug
        + std::fmt::Display
        + Default
        + Send
        + Binary
        + TryFrom<u8>
        + TryFrom<u16>
        + TryFrom<i8>
        + TryFrom<i16>
        + std::str::FromStr
        + PrimAs<u8>
        + PrimAs<u16>
        + PrimAs<u32>
        + PrimAs<u64>
        + PrimAs<u128>
        + PrimAs<usize>
        + PrimAs<i8>
        + PrimAs<i16>
        + PrimAs<i32>
        + PrimAs<i64>
        + PrimAs<i128>
        + PrimAs<isize>
        + PrimAs<f32>
        + PrimAs<f64>
        + PrimFrom<u8>
        + PrimFrom<u16>
        + PrimFrom<u32>
        + PrimFrom<u64>
        + PrimFrom<u128>
        + PrimFrom<usize>
        + PrimFrom<i8>
        + PrimFrom<i16>
        + PrimFrom<i32>
        + PrimFrom<i64>
        + PrimFrom<i128>
        + PrimFrom<isize>
        + PrimFrom<f32>
        + PrimFrom<f64>
        + 'static,
{
    /// The minimum value of this number. This will always be equivalent to the constant of the
    /// same name for the various number types. If you have the type of your number, you should
    /// probably use that constant instead.
    ///
    /// The use case for this is to be a consistant interface for when you do not know the type of
    /// the number.
    const MIN: Self;
    /// The maximum value of this number. This will always be equivalent to the constant of the
    /// same name for the various number types. If you have the type of your number, you should
    /// probably use that constant instead.
    ///
    /// The use case for this is to be a consistant interface for when you do not know the type of
    /// the number.
    const MAX: Self;
    const ZERO: Self;
    /// The number of bits that this number type takes up. For the integer types, they have a
    /// constant of the same name, but for floats you should probably just use number literals.
    ///
    /// The use case of this is for when you do not know the type of the number, only that it is a
    /// number.
    const BITS: u32;

    /// Works like max except that it assigns the result to self.
    /// ```no_run
    /// # use abes_nice_things::Number;
    /// # fn main() {
    /// # let mut num = 0;
    /// # let other_num = 5;
    /// num = num.max(other_num);
    /// // Is equivalent to
    /// num.max_assign(other_num);
    /// # }
    /// ```
    fn max_assign(&mut self, other: Self);

    /// Works like min except that it assigns the result to self.
    /// ```no_run
    /// # use abes_nice_things::Number;
    /// # fn main() {
    /// # let mut num = 0;
    /// # let other_num = 5;
    /// num = num.min(other_num);
    /// // Is equivalent to
    /// num.min_assign(other_num);
    /// # }
    /// ```
    fn min_assign(&mut self, other: Self);
}
/// This is a trait which is implemented
/// by all integers (i8-128 and u8-128)
/// so that you can use this as a generic
/// constraint. If you need more information,
/// look at [Number]
///
/// If you need a more specific constraint, consider [SignedInteger] or [UnsignedInteger]
pub unsafe trait Integer: Number
where
    Self: Ord
        + std::fmt::Binary
        + Eq
        + std::ops::Shl
        + std::ops::ShlAssign
        + std::ops::Shr
        + std::ops::ShrAssign
        + std::fmt::Binary
        + std::ops::BitAnd
        + for<'a> std::ops::BitAnd<&'a Self>
        + std::ops::BitAndAssign
        + for<'a> std::ops::BitAndAssign<&'a Self>
        + std::ops::BitOr
        + for<'a> std::ops::BitOr<&'a Self>
        + std::ops::BitOrAssign
        + for<'a> std::ops::BitOrAssign<&'a Self>
        + std::ops::BitXor
        + for<'a> std::ops::BitXor<&'a Self>
        + std::ops::BitXorAssign
        + for<'a> std::ops::BitXorAssign<&'a Self>
        + std::ops::Not
        + std::fmt::Octal
        + std::fmt::UpperHex
        + std::fmt::UpperExp
        + TryFrom<u32, Error: std::fmt::Debug>
        + TryFrom<u64, Error: std::fmt::Debug>
        + TryFrom<usize, Error: std::fmt::Debug>
        + TryFrom<u128, Error: std::fmt::Debug>
        + TryFrom<i32, Error: std::fmt::Debug>
        + TryFrom<i64, Error: std::fmt::Debug>
        + TryFrom<isize, Error: std::fmt::Debug>
        + TryFrom<i128, Error: std::fmt::Debug>
        + TryInto<u32, Error: std::fmt::Debug>
        + TryInto<u64, Error: std::fmt::Debug>
        + TryInto<usize, Error: std::fmt::Debug>
        + TryInto<u128, Error: std::fmt::Debug>
        + TryInto<i32, Error: std::fmt::Debug>
        + TryInto<i64, Error: std::fmt::Debug>
        + TryInto<isize, Error: std::fmt::Debug>
        + TryInto<i128, Error: std::fmt::Debug>,
{
}
/// This is a trait which is implemented
/// by all signed integers (i8-128) for
/// generic constraints. If you need more
/// information, look at [Number]
pub unsafe trait SignedInteger: Integer
where
    Self: std::ops::Neg + From<i8>,
{
    /// The unsigned version of this signed integer. So as an example, the unsigned version of
    /// [i32] is [u32]
    type Unsigned: UnsignedInteger<Signed = Self>;
}
/// This is a trait which is implemented
/// by all unsigned integers (u8-128) for
/// generic constraints. If you need more
/// information, look at [Number]
pub unsafe trait UnsignedInteger: Integer
where
    Self: From<u8>,
{
    /// The signed version of this unsigned integer.
    ///
    /// The signed version of [usize] is [isize] and so on.
    type Signed: SignedInteger<Unsigned = Self>;
}
/// This is a trait which is implemented
/// by both floats (f32 and f64) for
/// generic constraints. If you need more
/// information, look at [Number]
pub unsafe trait Float: Number
where
    Self: std::ops::Neg + From<f32> + TryInto<f64, Error: std::fmt::Debug>,
    for<'a> &'a Self: std::ops::Neg,
{
}
/// A trait which is implemented by every number that has an atomic variant. (hint, the floats
/// don't have atomics)
pub unsafe trait HasAtomic: Integer {
    /// The atomic version of this integer.
    /// [usize] -> [AtomicUsize](std::sync::atomic::AtomicUsize) and so on.
    type Atomic: AtomicInteger<Counterpart = Self>;
}
/// A trait implemented by all atomic integers. It has some methods for doing atomic operations
/// which is nice. Might add more later but I am lazy and won't until someone needs them.
pub unsafe trait AtomicInteger
where
    Self: Sized
        + Send
        + Sync
        + From<Self::Counterpart>
        + std::fmt::Debug
        + Default
        + std::panic::RefUnwindSafe
        + Binary
        + 'static,
{
    /// The non-atomic version of this atomic integer.
    /// [AtomicU8](std::sync::atomic::AtomicU8) -> [u8] and so forth.
    type Counterpart: HasAtomic<Atomic = Self>;
    /// The load operation for this atomic integer, if you do not know what that means, you should
    /// not be working with atomics and should be reading docs instead.
    fn load(&self, ordering: Ordering) -> Self::Counterpart;
    /// The store operation for this atomic integer, if you do not know what that means, then read
    /// the docs on atomics.
    fn store(&self, val: Self::Counterpart, ordering: Ordering);
}
/// A trait for primitive type conversions using the [as] keyword. This works nicely for converting
/// from unknown [Number] types to known numbers.
/// ```
/// # use abes_nice_things::{Number, PrimAs};
/// # fn main() { assert_eq!(example(7_i8), 7_usize); }
/// fn example(num: impl Number) -> usize {
///     return num.prim_as()
/// }
/// ```
/// If you want to convert from a known number to an unknown [Number], then try using [PrimFrom]
///
/// Fun fact: Every number type can be converted to from any number in the range 0..128
pub trait PrimAs<T> {
    /// A primitive type conversion into a known [Number] using the [as] keyword. If you have
    /// access to the actual number type, then you should probably just use the [as] keyword. But
    /// if you are dealing with an unknown [Number] then this will let you coerce it into a known
    /// type.
    fn prim_as(self) -> T;
}
/// A trait for primitive type conversions from known number types using the [as] keyword. An
/// example of a situation where this is useful is when you want to increment an unknown [Number]
/// by 1.
/// ```
/// # use abes_nice_things::{PrimFrom, Number};
/// # fn main() { let mut num = 3; increment(&mut num); assert_eq!(num, 4); }
/// fn increment<N: Number>(number: &mut N) {
///     *number += N::prim_from(1);
/// }
/// ```
pub trait PrimFrom<T> {
    /// A primitive conversion to an unknown [Number] type from a known number primitive. If you do
    /// not know what those mean, then you should read more docs and come back, or you should not
    /// be using this.
    fn prim_from(src: T) -> Self;
}
macro_rules! prim_as_helper_helper {
    ($type:ty, $($into:ty)*) => {
        $(
            impl PrimAs<$into> for $type {
                fn prim_as(self) -> $into {
                    self as $into
                }
            }
            impl PrimFrom<$into> for $type {
                fn prim_from(src: $into) -> $type {
                    src as $type
                }
            }
        )*
    }
}
macro_rules! prim_as_helper {
    ($type:ty) => {
        prim_as_helper_helper!($type, u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize f32 f64);
    }
}
macro_rules! number_trait_helper_helper {
    ($type:ty) => {
        const MIN: $type = <$type>::MIN;
        const MAX: $type = <$type>::MAX;
        const ZERO: $type = 0 as $type;
        fn max_assign(&mut self, other: $type) {
            *self = (*self).max(other);
        }
        fn min_assign(&mut self, other: $type) {
            *self = (*self).min(other);
        }
    };
}
macro_rules! number_trait_helper {
    ($type:ty) => {
        unsafe impl Number for $type {
            number_trait_helper_helper!($type);
            const BITS: u32 = <$type>::BITS;
        }
        prim_as_helper!($type);
    };
    ($type:ty, $bits:literal) => {
        unsafe impl Number for $type {
            number_trait_helper_helper!($type);
            const BITS: u32 = $bits;
        }
        prim_as_helper!($type);
    };
}
macro_rules! integer_trait_helper {
    ($type:ty) => {
        number_trait_helper!($type);
        unsafe impl Integer for $type {}
    };
    ($($signed:ty = $unsigned:ty),*) => {
        $(
            integer_trait_helper!($signed);
            integer_trait_helper!($unsigned);
            unsafe impl SignedInteger for $signed {
                type Unsigned = $unsigned;
            }
            unsafe impl UnsignedInteger for $unsigned {
                type Signed = $signed;
            }
        )*
    }
}
macro_rules! atomic_helper {
    ($type:ty, $atomic:ty) => {
        unsafe impl HasAtomic for $type {
            type Atomic = $atomic;
        }
        unsafe impl AtomicInteger for $atomic {
            type Counterpart = $type;
            fn load(&self, ordering: Ordering) -> Self::Counterpart {
                self.load(ordering)
            }
            fn store(&self, val: Self::Counterpart, ordering: Ordering) {
                self.store(val, ordering)
            }
        }
    };
    ($($type:ty = $atomic:ty),*) => {
        $(atomic_helper!($type, $atomic);)*
    }
}
macro_rules! float_trait_helper {
    ($($type:ty = $bits:literal),*) => {
        $(
            number_trait_helper!($type, $bits);
            unsafe impl Float for $type {}
        )*
    };
}
integer_trait_helper!(
    i8 = u8,
    i16 = u16,
    i32 = u32,
    i64 = u64,
    i128 = u128,
    isize = usize
);
atomic_helper!(
    u8 = AtomicU8,
    u16 = AtomicU16,
    u32 = AtomicU32,
    u64 = AtomicU64,
    usize = AtomicUsize,
    i8 = AtomicI8,
    i16 = AtomicI16,
    i32 = AtomicI32,
    i64 = AtomicI64,
    isize = AtomicIsize
);
float_trait_helper!(f32 = 32, f64 = 64);
