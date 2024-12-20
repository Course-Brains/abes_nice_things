use std::io::{Read, Write};
/// This trait is designed to allow for easier conversion from binary
/// in a defined and consistent way.
pub trait FromBinary {
    fn from_binary(binary: &mut dyn Read) -> Self;
}
pub trait FromBinarySized where Self: FromBinary {
    const LEN: usize;
}
pub trait ToBinary {
    fn to_binary(self, write: &mut dyn Write);
}
macro_rules! num_helper {
    ($type: ty) => {
        impl FromBinarySized for $type {
            const LEN: usize = std::mem::size_of::<$type>();
        }
        impl FromBinary for $type {
            fn from_binary(binary: &mut dyn Read) -> Self {
                let mut chunk: [u8; Self::LEN] = [0; Self::LEN];
                binary.read_exact(&mut chunk).unwrap();
                <$type>::from_le_bytes(chunk)
            }
        }
        impl ToBinary for $type {
            fn to_binary(self, write: &mut dyn Write) {
                write.write_all(&self.to_le_bytes()).unwrap()
            }
        }
    }
}
num_helper!(u8);
num_helper!(u16);
num_helper!(u32);
num_helper!(u64);
num_helper!(u128);
num_helper!(usize);
num_helper!(i8);
num_helper!(i16);
num_helper!(i32);
num_helper!(i64);
num_helper!(i128);
num_helper!(isize);
num_helper!(f32);
num_helper!(f64);
impl FromBinary for bool {
    fn from_binary(binary: &mut dyn Read) -> Self {
        let mut buf: [u8; 1] = [0];
        binary.read_exact(&mut buf).unwrap();
        match buf[0] {
            0b0000_0000 => {
                false
            }
            0b0000_0001 => {
                true
            }
            _ => {
                panic!("Expected bool byte but found: {}", buf[0])
            }
        }
    }
}
impl FromBinarySized for bool {
    const LEN: usize = 1;
}
impl ToBinary for bool {
    fn to_binary(self, write: &mut dyn Write) {
        match self {
            true => {
                write.write_all(&[0b0000_0001]).unwrap()
            }
            false => {
                write.write_all(&[0b0000_0000]).unwrap()
            }
        }
    }
}
impl FromBinary for String {
    fn from_binary(binary: &mut dyn Read) -> Self {
        let len = usize::from_binary(binary);
        let mut buf = vec![0; len];
        binary.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
impl ToBinary for String {
    fn to_binary(self, write: &mut dyn Write) {
        self.len().to_binary(write);
        write.write_all(self.as_bytes()).unwrap();
    }
}
impl<T: FromBinary> FromBinary for Vec<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        let len = usize::from_binary(binary);
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(T::from_binary(binary));
        }
        return out
    }
}
impl<T: ToBinary> ToBinary for Vec<T> {
    fn to_binary(self, write: &mut dyn Write) {
        self.len().to_binary(write);
        for i in self {
            i.to_binary(write)
        }
    }
}
impl<T: FromBinary> FromBinary for Option<T> {
    // Option<T> is not FromBinarySized because it only takes up
    // 1 byte if it is a None, but takes T::LEN+1 otherwise
    fn from_binary(binary: &mut dyn Read) -> Self {
        match bool::from_binary(binary) {
            true => {// Some(T)
                Some(T::from_binary(binary))
            }
            false => {// None
                None
            }
        }
    }
}
impl<T: ToBinary> ToBinary for Option<T> {
    fn to_binary(self, write: &mut dyn Write) {
        match self {
            Some(value) => {
                true.to_binary(write);
                value.to_binary(write);
            }
            None => {
                false.to_binary(write)
            }
        }
    }
}