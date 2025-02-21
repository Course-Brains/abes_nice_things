use std::io::{Read, Write};
/// This trait is designed to allow for easier conversion from binary
/// in a defined and consistent way.
pub trait FromBinary {
    fn from_binary(binary: &mut dyn Read) -> Self;
}
pub trait ToBinary {
    fn to_binary(&self, binary: &mut dyn Write);
}
/// A convinience trait which is implemented for
/// everything that implements both [FromBinary]
/// and [ToBinary], meaning that instead of having
/// your type restrictions be
/// ```ignore
/// impl<T: FromBinary + ToBinary> Thingamajig { ... }
/// ```
/// You can instead have the much shorter
/// ```ignore
/// impl<T: Binary> Thingamajig { ... }
/// ```
pub trait Binary where Self: FromBinary + ToBinary {}
impl<T: FromBinary + ToBinary> Binary for T {}
macro_rules! num_helper {
    ($($type: ty)*) => {
        $(
            impl FromBinary for $type {
                fn from_binary(binary: &mut dyn Read) -> Self {
                    let mut chunk: [u8; std::mem::size_of::<Self>()] = [0; std::mem::size_of::<Self>()];
                    binary.read_exact(&mut chunk).unwrap();
                    <$type>::from_le_bytes(chunk)
                }
            }
            impl ToBinary for $type {
                fn to_binary(&self, binary: &mut dyn Write) {
                    binary.write_all(&self.to_le_bytes()).unwrap()
                }
            }
        )*
    }
}
num_helper!(u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64);
#[cfg(feature = "dyn_binary")]
num_helper!(usize isize);
impl FromBinary for std::primitive::char {
    fn from_binary(binary: &mut dyn Read) -> Self {
        char::from_u32(u32::from_binary(binary)).unwrap()
    }
}
impl ToBinary for std::primitive::char {
    fn to_binary(&self, binary: &mut dyn Write) {
        (*self as u32).to_binary(binary)
    }
}
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
impl ToBinary for bool {
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            true => {
                binary.write_all(&[0b0000_0001]).unwrap()
            }
            false => {
                binary.write_all(&[0b0000_0000]).unwrap()
            }
        }
    }
}
impl FromBinary for String {
    fn from_binary(binary: &mut dyn Read) -> Self {
        #[cfg(feature = "dyn_binary")]
        let len = usize::from_binary(binary);
        #[cfg(not(feature = "dyn_binary"))]
        let len = u32::from_binary(binary) as usize;
        let mut buf = vec![0; len];
        binary.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
impl ToBinary for String {
    fn to_binary(&self, binary: &mut dyn Write) {
        #[cfg(feature = "dyn_binary")]
        self.len().to_binary(binary);
        #[cfg(not(feature = "dyn_binary"))]
        (self.len() as u32).to_binary(binary);
        binary.write_all(self.as_bytes()).unwrap();
    }
}
impl<T: FromBinary> FromBinary for Vec<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        #[cfg(feature = "dyn_binary")]
        let len = usize::from_binary(binary);
        #[cfg(not(feature = "dyn_binary"))]
        let len = u32::from_binary(binary) as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(T::from_binary(binary));
        }
        return out
    }
}
impl<T: ToBinary> ToBinary for Vec<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        #[cfg(feature = "dyn_binary")]
        self.len().to_binary(binary);
        #[cfg(not(feature = "dyn_binary"))]
        (self.len() as u32).to_binary(binary);
        for i in self {
            i.to_binary(binary)
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
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            Some(value) => {
                true.to_binary(binary);
                value.to_binary(binary);
            }
            None => {
                false.to_binary(binary)
            }
        }
    }
}
impl<T: FromBinary, E: FromBinary> FromBinary for Result<T, E> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        match u8::from_binary(binary) {
            0 => Ok(T::from_binary(binary)),
            1 => Err(E::from_binary(binary)),
            _ => unreachable!("Zoinks! It's the gay blade!")
        }
    }
}
impl<T: ToBinary, E: ToBinary> ToBinary for Result<T, E> {
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            Ok(value) => {
                0_u8.to_binary(binary);
                value.to_binary(binary);
            }
            Err(error) => {
                1_u8.to_binary(binary);
                error.to_binary(binary);
            }
        }
    }
}
impl FromBinary for () {
    fn from_binary(_binary: &mut dyn Read) -> Self {
        ()
    }
}
impl ToBinary for () {
    fn to_binary(&self, _binary: &mut dyn Write) {}
}
impl<T: FromBinary, const N: usize> FromBinary for [T; N] {
    fn from_binary(binary: &mut dyn Read) -> Self {
        let mut out = [const { None }; N];
        for index in 0..N {
            out[index] = Some(T::from_binary(binary))
        }
        return out.map(|x| x.unwrap())
    }
}
impl<T: ToBinary, const N: usize> ToBinary for [T; N] {
    fn to_binary(&self, binary: &mut dyn Write) {
        for value in self.into_iter() {
            value.to_binary(binary);
        }
    }
}
impl FromBinary for std::cmp::Ordering {
    fn from_binary(binary: &mut dyn Read) -> Self {
        match u8::from_binary(binary) {
            0 => std::cmp::Ordering::Equal,
            1 => std::cmp::Ordering::Less,
            2 => std::cmp::Ordering::Greater,
            _ => unreachable!("RUH ROH RAGGY")
        }
    }
}
impl ToBinary for std::cmp::Ordering {
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            std::cmp::Ordering::Equal => 0_u8.to_binary(binary),
            std::cmp::Ordering::Less => 1_u8.to_binary(binary),
            std::cmp::Ordering::Greater => 2_u8.to_binary(binary)
        }
    }
}
impl<T: FromBinary> FromBinary for Box<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Box::new(T::from_binary(binary))
    }
}
impl<T: ToBinary> ToBinary for Box<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.as_ref().to_binary(binary);
    }
}
impl FromBinary for std::net::Ipv4Addr {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::from_bits(u32::from_binary(binary))
    }
}
impl ToBinary for std::net::Ipv4Addr {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.to_bits().to_binary(binary)
    }
}
impl FromBinary for std::net::Ipv6Addr {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::from_bits(u128::from_binary(binary))
    }
}
impl ToBinary for std::net::Ipv6Addr {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.to_bits().to_binary(binary)
    }
}
impl FromBinary for std::net::IpAddr {
    fn from_binary(binary: &mut dyn Read) -> Self {
        match u8::from_binary(binary) {
            0 => std::net::IpAddr::V4(std::net::Ipv4Addr::from_binary(binary)),
            1 => std::net::IpAddr::V6(std::net::Ipv6Addr::from_binary(binary)),
            _ => unreachable!("AW NAWR")
        }
    }
}
impl ToBinary for std::net::IpAddr {
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            std::net::IpAddr::V4(addr) => {
                0_u8.to_binary(binary);
                addr.to_binary(binary);
            }
            std::net::IpAddr::V6(addr) => {
                1_u8.to_binary(binary);
                addr.to_binary(binary);
            }
        }
    }
}
impl FromBinary for std::net::SocketAddrV4 {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::new(
            std::net::Ipv4Addr::from_binary(binary),
            u16::from_binary(binary)
        )
    }
}
impl ToBinary for std::net::SocketAddrV4 {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.ip().to_binary(binary);
        self.port().to_binary(binary);
    }
}
impl FromBinary for std::net::SocketAddrV6 {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::new(
            std::net::Ipv6Addr::from_binary(binary),
            u16::from_binary(binary),
            u32::from_binary(binary),
            u32::from_binary(binary)
        )
    }
}
impl ToBinary for std::net::SocketAddrV6 {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.ip().to_binary(binary);
        self.port().to_binary(binary);
        self.flowinfo().to_binary(binary);
        self.scope_id().to_binary(binary);
    }
}
impl FromBinary for std::net::SocketAddr {
    fn from_binary(binary: &mut dyn Read) -> Self {
        match u8::from_binary(binary) {
            0 => std::net::SocketAddr::V4(std::net::SocketAddrV4::from_binary(binary)),
            1 => std::net::SocketAddr::V6(std::net::SocketAddrV6::from_binary(binary)),
            _ => unreachable!("Zoinks!")
        }
    }
}
impl ToBinary for std::net::SocketAddr {
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            std::net::SocketAddr::V4(addr) => {
                0_u8.to_binary(binary);
                addr.to_binary(binary)
            }
            std::net::SocketAddr::V6(addr) => {
                1_u8.to_binary(binary);
                addr.to_binary(binary)
            }
        }
    }
}
impl<T: FromBinary> FromBinary for std::ops::Bound<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        match u8::from_binary(binary) {
            0 => std::ops::Bound::Excluded(T::from_binary(binary)),
            1 => std::ops::Bound::Included(T::from_binary(binary)),
            2 => std::ops::Bound::Unbounded,
            _ => unreachable!("Let's split up, gang!")
        }
    }
}
impl<T: ToBinary> ToBinary for std::ops::Bound<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            std::ops::Bound::Excluded(point) => {
                0_u8.to_binary(binary);
                point.to_binary(binary)
            }
            std::ops::Bound::Included(point) => {
                1_u8.to_binary(binary);
                point.to_binary(binary)
            }
            std::ops::Bound::Unbounded => 2_u8.to_binary(binary)
        }
    }
}
impl<T: FromBinary> FromBinary for std::ops::Range<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        T::from_binary(binary)..T::from_binary(binary)
    }
}
impl<T: ToBinary> ToBinary for std::ops::Range<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.start.to_binary(binary);
        self.end.to_binary(binary);
    }
}
impl<T: FromBinary> FromBinary for std::ops::RangeFrom<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        T::from_binary(binary)..
    }
}
impl<T: ToBinary> ToBinary for std::ops::RangeFrom<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.start.to_binary(binary)
    }
}
impl<T: FromBinary> FromBinary for std::ops::RangeInclusive<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        T::from_binary(binary)..=T::from_binary(binary)
    }
}
impl<T: ToBinary> ToBinary for std::ops::RangeInclusive<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.start().to_binary(binary);
        self.end().to_binary(binary);
    }
}
impl<T: FromBinary> FromBinary for std::ops::RangeTo<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        ..T::from_binary(binary)
    }
}
impl<T: ToBinary> ToBinary for std::ops::RangeTo<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.end.to_binary(binary)
    }
}
impl<T: FromBinary> FromBinary for std::ops::RangeToInclusive<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        ..=T::from_binary(binary)
    }
}
impl<T: ToBinary> ToBinary for std::ops::RangeToInclusive<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.end.to_binary(binary)
    }
}
// Cannot implement it for Instant because rust is mean and a bully :(
// aka. I cannot access the underlying numbers used to store the
// Instant meaning that I cannot convert them to/from binary
impl FromBinary for std::time::Duration {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::new(
            u64::from_binary(binary),
            u32::from_binary(binary)
        )
    }
}
impl ToBinary for std::time::Duration {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.as_secs().to_binary(binary);
        self.subsec_nanos().to_binary(binary);
    }
}
/*impl FromBinary for std:: {
    fn from_binary(binary: &mut dyn Read) -> Self {
        
    }
}*/
#[cfg(test)]
mod tests {
    use super::{FromBinary, ToBinary};
    use std::collections::VecDeque;
    mod numbers {
        use super::{FromBinary, ToBinary};
        use std::collections::VecDeque;
        macro_rules! num_helper {
            ($type:ty, $name:ident) => {
                #[test]
                fn $name() {
                    for i in <$type>::MIN..=<$type>::MAX {
                        let check = i;
                        let mut binary = VecDeque::new();
                        i.to_binary(&mut binary);
                        assert_eq!(check, <$type>::from_binary(&mut binary), "Failed at number: {i}")
                    }
                }
            };
            ($type:ty, $name:ident,) => {
                #[test]
                #[ignore]
                fn $name() {
                    for i in <$type>::MIN..=<$type>::MAX {
                        let check = i;
                        let mut binary = VecDeque::new();
                        i.to_binary(&mut binary);
                        assert_eq!(check, <$type>::from_binary(&mut binary), "Failed at number: {i}")
                    }
                }
            }
        }
        num_helper!(u8, u8);
        num_helper!(u16, u16);
        num_helper!(u32, u32,);
        num_helper!(u64, u64,);
        num_helper!(u128, u128,);
        #[cfg(feature = "dyn_binary")]
        num_helper!(usize, usize,);
        num_helper!(i8, i8,);
        num_helper!(i16, i16,);
        num_helper!(i32, i32,);
        num_helper!(i64, i64,);
        num_helper!(i128, i128,);
        #[cfg(feature = "dyn_binary")]
        num_helper!(isize, isize);
    }
    mod bool {
        use super::{FromBinary, ToBinary, VecDeque};
        #[test]
        fn normal_true() {
            let mut binary = VecDeque::new();
            true.to_binary(&mut binary);
            assert_eq!(true, bool::from_binary(&mut binary));
        }
        #[test]
        fn normal_false() {
            let mut binary = VecDeque::new();
            false.to_binary(&mut binary);
            assert_eq!(false, bool::from_binary(&mut binary));
        }
        #[test]
        fn inequal_true() {
            let mut binary = VecDeque::new();
            true.to_binary(&mut binary);
            assert_ne!(false, bool::from_binary(&mut binary));
        }
        #[test]
        fn inequal_false() {
            let mut binary = VecDeque::new();
            false.to_binary(&mut binary);
            assert_ne!(true, bool::from_binary(&mut binary));
        }
    }
}