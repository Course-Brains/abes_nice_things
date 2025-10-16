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
        + std::ops::Add
        + for<'a> std::ops::Add<&'a Self>
        + std::ops::AddAssign
        + for<'a> std::ops::AddAssign<&'a Self>
        + std::ops::Sub
        + for<'a> std::ops::Sub<&'a Self>
        + std::ops::SubAssign
        + for<'a> std::ops::SubAssign<&'a Self>
        + std::ops::Mul
        + for<'a> std::ops::Mul<&'a Self>
        + std::ops::MulAssign
        + for<'a> std::ops::MulAssign<&'a Self>
        + std::ops::Div
        + for<'a> std::ops::Div<&'a Self>
        + std::ops::DivAssign
        + for<'a> std::ops::DivAssign<&'a Self>
        + std::ops::Rem
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
        + std::str::FromStr,
{
    /// Gets the minimum value for a number. This is equivalent to using the MIN constant for each
    /// number primitive:
    /// ```
    /// # use abes_nice_things::Number;
    /// # fn main() {
    /// assert_eq!(isize::get_min(), isize::MIN)
    /// # }
    /// ```
    /// In almost every case, it is better to just use the constant. However, in cases where you do
    /// not know which number primitive you are handling and therefore cannot use the constant,
    /// this will fill that role.
    fn get_min() -> Self;

    /// Gets the maximum value for a number. This is equivalent to using the MAX constant for each
    /// number primitive:
    /// ```
    /// # use abes_nice_things::Number;
    /// # fn main() {
    /// assert_eq!(isize::get_max(), isize::MAX)
    /// # }
    /// ```
    /// In almost every case, it is better to just use the constant. However, in cases where you do
    /// not know which number primitive you are handling and therefore cannot use the constant,
    /// this will fill that role.
    fn get_max() -> Self;

    /// Gets the number of bits for a given number. This is usually equivalent to using the BITS
    /// constant for the relevant type, but this will work even if you do not know which number
    /// primitive you are handling.
    /// ```
    /// # use abes_nice_things::Number;
    /// # fn main() {
    /// assert_eq!(usize::get_bits(), usize::BITS);
    /// # }
    /// ```
    fn get_bits() -> u32;

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
        + TryFrom<u32>
        + TryFrom<u64>
        + TryFrom<usize>
        + TryFrom<u128>
        + TryFrom<i32>
        + TryFrom<i64>
        + TryFrom<isize>
        + TryFrom<i128>,
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
    for<'a> &'a Self: std::ops::BitAnd<Self>
        + std::ops::BitAnd<&'a Self>
        + std::ops::BitOr<Self>
        + std::ops::BitOr<&'a Self>
        + std::ops::BitXor<Self>
        + std::ops::BitXor<&'a Self>,
{
}
/// This is a trait which is implemented
/// by both floats (f32 and f64) for
/// generic constraints. If you need more
/// information, look at [Number]
pub unsafe trait Float: Number
where
    Self: std::ops::Neg + From<f32>,
    for<'a> &'a Self: std::ops::Neg,
{
}
macro_rules! number_trait_helper_helper {
    ($type:ty) => {
        fn get_min() -> $type {
            <$type>::MIN
        }
        fn get_max() -> $type {
            <$type>::MAX
        }
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
            fn get_bits() -> u32 {
                <$type>::BITS
            }
        }
    };
    ($type:ty, $bits:literal) => {
        unsafe impl Number for $type {
            number_trait_helper_helper!($type);
            fn get_bits() -> u32 {
                $bits
            }
        }
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
