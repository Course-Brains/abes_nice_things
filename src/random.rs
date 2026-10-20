use crate::from_binary::Result;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
thread_local! {
    pub static URANDOM: RefCell<BufReader<File>> =
        RefCell::new(BufReader::new(File::open("/dev/urandom").unwrap()));
    pub static RANDOM: RefCell<BufReader<File>> = RefCell::new(BufReader::new(File::open("/dev/random").unwrap()));
}
/// Creates a uniform random number between 1 and 2
///
/// Specifically this creates a 64 bit floating point number with an exponent of 0, a positive
/// sign and random data in the mantissa
///
/// This uses /dev/urandom which means that if there is not sufficient data to create randomness it
/// will not block and is less random because of that
pub fn urandom() -> f64 {
    URANDOM
        .with_borrow_mut(|urandom| random_from_source(urandom))
        .unwrap()
}
/// Creates a uniform random number between 1 and 2
///
/// Specifically this creates a 64 bit floating point number with an exponent of 0, a positive
/// sign and random data in the mantissa
///
/// this uses /dev/random which means that if there is insufficient data to create random it will
/// block until there is enough so it will always be random but may not always be nonblocking
pub fn random() -> f64 {
    RANDOM
        .with_borrow_mut(|urandom| random_from_source(urandom))
        .unwrap()
}
pub fn random_from_source(source: &mut dyn Read) -> Result<f64> {
    let mut mantissa = [0; 7];
    source.read_exact(&mut mantissa)?;
    let mut bits: u64 = 0b00111111_11110000_00000000_00000000_00000000_00000000_00000000_00000000;
    //                    ^ sign
    //                     ^^^^^^^ ^^^^ exponent
    // 52 bits of mantissa
    // nearest greater multiple of 8 is 56 aka 7 bytes
    let mask: u64 = 0b00000000_00001111_11111111_11111111_11111111_11111111_11111111_11111111;
    let mut mantissa_bits = [0; 8];
    for i in 0..7 {
        mantissa_bits[i + 1] = mantissa[i]
    }
    let mantissa = u64::from_le_bytes(mantissa_bits);
    let mantissa = mantissa & mask;
    bits |= mantissa;

    Ok(f64::from_bits(bits))
}

/// Create a random instance of self
pub trait Random: Sized {
    fn random_from_source(source: &mut dyn Read) -> Result<Self>;

    /// Get a random value from all possible states of the type. Because it is for all possible
    /// states, floats do not implement this. If you want a float, try [random].
    /// This uses /dev/random so it will always be random but may block.
    fn random() -> Self {
        RANDOM
            .with_borrow_mut(|random| Self::random_from_source(random))
            .unwrap()
    }
    /// Read the other method and look up /dev/urandom
    fn urandom() -> Self {
        URANDOM
            .with_borrow_mut(|urandom| Self::random_from_source(urandom))
            .unwrap()
    }
}
impl Random for bool {
    fn random_from_source(source: &mut dyn Read) -> Result<Self> {
        Ok((u8::random_from_source(source)? & 0b1) == 1)
    }
}
impl<T: Random, const N: usize> Random for [T; N] {
    fn random_from_source(source: &mut dyn Read) -> Result<Self> {
        // SAFETY: We are overwriting all of these before returning and never reading them
        let mut out = [const { std::mem::MaybeUninit::zeroed() }; N];
        for index in 0..N {
            out[index].write(T::random_from_source(source)?);
        }
        // SAFETY: We literally just set these with valid values
        Ok(out.map(|element| unsafe { element.assume_init() }))
    }
}
// for plain old data which covers every possible value
macro_rules! random_num_helper {
    ($type:ty) => {
        impl Random for $type {
            fn random_from_source(source: &mut dyn Read) -> Result<Self> {
                let mut buf = [0; std::mem::size_of::<$type>()];
                source.read_exact(&mut buf)?;
                Ok(Self::from_le_bytes(buf))
            }
        }
    };
    ($($type:ty)*) => {
        $(random_num_helper!($type);)*
    };
}
random_num_helper!(u8 u16 u32 u64 u128 usize i8 i16 i32 i64 i128 isize);

/// Create a random value from self
pub trait PickRandom<T> {
    fn generate(&self) -> T {
        RANDOM
            .with_borrow_mut(|random| self.generate_from_source(random))
            .unwrap()
    }
    fn generate_urandom(&self) -> T {
        URANDOM
            .with_borrow_mut(|urandom| self.generate_from_source(urandom))
            .unwrap()
    }
    fn generate_from_source(&self, source: &mut dyn Read) -> Result<T>;
}
impl<'a, T> PickRandom<&'a T> for &'a [T] {
    fn generate_from_source(&self, source: &mut dyn Read) -> Result<&'a T> {
        Ok(&self[(..self.len()).generate_from_source(source)?])
    }
}
impl PickRandom<usize> for std::ops::Range<usize> {
    fn generate_from_source(&self, source: &mut dyn Read) -> Result<usize> {
        Ok(
            ((random_from_source(source)? - 1.0) * (self.end - self.start) as f64
                + self.start as f64) as usize,
        )
    }
}
impl PickRandom<usize> for std::ops::RangeInclusive<usize> {
    fn generate_from_source(&self, source: &mut dyn Read) -> Result<usize> {
        (*self.start()..*self.end() + 1).generate_from_source(source)
    }
}
impl PickRandom<usize> for std::ops::RangeTo<usize> {
    fn generate_from_source(&self, source: &mut dyn Read) -> Result<usize> {
        Ok(((random_from_source(source)? - 1.0) * self.end as f64) as usize)
    }
}
