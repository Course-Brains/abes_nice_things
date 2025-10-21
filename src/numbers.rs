use crate::Binary;
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
        + PrimFrom<f64>,
{
    const MIN: Self;
    const MAX: Self;
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
pub unsafe trait Integer: Number
where
    Self: Ord
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
    for<'a> &'a Self: std::ops::Neg
        + std::ops::BitAnd<Self>
        + std::ops::BitAnd<&'a Self>
        + std::ops::BitOr<Self>
        + std::ops::BitOr<&'a Self>
        + std::ops::BitXor<Self>
        + std::ops::BitXor<&'a Self>,
{
}
/// This is a trait which is implemented
/// by all unsigned integers (u8-128) for
/// generic constraints. If you need more
/// information, look at [Number]
pub unsafe trait UnsignedInteger: Integer
where
    Self: From<u8>,
{
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
pub trait PrimAs<T> {
    fn prim_as(self) -> T;
}
pub trait PrimFrom<T> {
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
}
macro_rules! signed_integer_trait_helper {
    ($($type:ty)*) => {
        $(
            integer_trait_helper!($type);
            unsafe impl SignedInteger for $type {}
        )*
    }
}
macro_rules! unsigned_integer_trait_helper {
    ($($type:ty)*) => {
        $(
            integer_trait_helper!($type);
            unsafe impl UnsignedInteger for $type {}
        )*
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
unsigned_integer_trait_helper!(u8 u16 u32 u64 u128 usize);
signed_integer_trait_helper!(i8 i16 i32 i64 i128 isize);
float_trait_helper!(f32 = 32, f64 = 64);
