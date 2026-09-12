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
    let mut buf = [0; 7];
    URANDOM.with_borrow_mut(|urandom| urandom.read_exact(&mut buf).unwrap());
    f64_from_bytes(buf)
}
/// Creates a uniform random number between 1 and 2
///
/// Specifically this creates a 64 bit floating point number with an exponent of 0, a positive
/// sign and random data in the mantissa
///
/// this uses /dev/random which means that if there is insufficient data to create random it will
/// block until there is enough so it will always be random but may not always be nonblocking
pub fn random() -> f64 {
    let mut buf = [0; 7];
    RANDOM.with_borrow_mut(|urandom| urandom.read_exact(&mut buf).unwrap());
    f64_from_bytes(buf)
}
fn f64_from_bytes(mantissa: [u8; 7]) -> f64 {
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

    f64::from_bits(bits)
}

/// Create a random instance of self
pub trait Random {
    /// Get a random value from all possible states of the type. Because it is for all possible
    /// states, floats do not implement this. If you want a float, try [random].
    /// This uses /dev/random so it will always be random but may block.
    fn random() -> Self;
    /// Read the other method and look up /dev/urandom
    fn urandom() -> Self;
}
impl Random for bool {
    fn random() -> Self {
        (u8::random() & 1) == 1
    }
    fn urandom() -> Self {
        (u8::urandom() & 1) == 1
    }
}
// for plain old data which covers every possible value
macro_rules! random_num_helper {
    ($type:ty) => {
        impl Random for $type {
            fn random() -> Self {
                RANDOM.with_borrow_mut(|random| {
                    let mut buf = [0; std::mem::size_of::<$type>()];
                    random.read_exact(&mut buf).unwrap();
                    Self::from_le_bytes(buf)
                })
            }
            fn urandom() -> Self {
                URANDOM.with_borrow_mut(|urandom| {
                    let mut buf = [0; std::mem::size_of::<$type>()];
                    urandom.read_exact(&mut buf).unwrap();
                    Self::from_le_bytes(buf)
                })
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
    fn generate(&self) -> T;
    fn generate_urandom(&self) -> T;
}
impl<'a, T> PickRandom<&'a T> for &'a [T] {
    fn generate(&self) -> &'a T {
        &self[(..self.len()).generate()]
    }
    fn generate_urandom(&self) -> &'a T {
        &self[(..self.len()).generate_urandom()]
    }
}
impl PickRandom<usize> for std::ops::Range<usize> {
    fn generate(&self) -> usize {
        ((random() - 1.0) * (self.end - self.start) as f64 + self.start as f64) as usize
    }
    fn generate_urandom(&self) -> usize {
        ((urandom() - 1.0) * (self.end - self.start) as f64 + self.start as f64) as usize
    }
}
impl PickRandom<usize> for std::ops::RangeInclusive<usize> {
    fn generate(&self) -> usize {
        (*self.start()..*self.end() + 1).generate()
    }
    fn generate_urandom(&self) -> usize {
        (*self.start()..*self.end() + 1).generate_urandom()
    }
}
impl PickRandom<usize> for std::ops::RangeTo<usize> {
    fn generate(&self) -> usize {
        ((random() - 1.0) * self.end as f64) as usize
    }
    fn generate_urandom(&self) -> usize {
        ((urandom() - 1.0) * self.end as f64) as usize
    }
}
