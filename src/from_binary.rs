use std::io::{Read, Write};
use std::ops::Deref;
#[cfg(feature = "transmute_binary")]
use transmuters::*;
#[cfg(feature = "transmute_binary")]
use std::mem::transmute;
/// This trait is designed to allow for easier conversion from binary
/// in a defined and consistent way.
/// It contains a singular method([from_binary](FromBinary::from_binary))
/// and is designed exlusively to convert data from binary.
/// For more information on usage, see [from_binary](FromBinary::from_binary).
/// 
/// # Implementation
/// This is designed to be as easily expandable as possible.
/// A large part of that is due to the method taking a [Reader](Read).
/// Because [Readers](Read) will have the cursor move after
/// each read, the actual implementation is very easy.
/// ```
/// # use abes_nice_things::FromBinary;
/// use std::io::Read;
/// struct Data {
///     num: i64,
///     vec: Vec<String>,
///     option: Option<bool>
/// }
/// impl FromBinary for Data {
///     fn from_binary(binary: &mut dyn Read) -> Self {
///         Data {
///             num: i64::from_binary(binary),
///             vec: Vec::from_binary(binary),
///             option: Option::from_binary(binary)
///             // Notice how the generics were not included
///         }
///     }
/// }
/// ```
/// As you hopefully noticed, the generics for the [Vec]
/// and [Option] were not included before the method call.
/// The only important aspect for the conversion is that
/// the [from_binary](FromBinary::from_binary) conversion
/// and the [to_binary](ToBinary::to_binary) conversion
/// [read](Read)/[write](Write) in the same order.
/// Meaning that in this case, because we [read](Read) the data
/// in the order of "num" -> "vec" -> "option" we have
/// to also [write](Write) in the order of "num" ->
/// "vec" -> "option" not the opposite.
/// # Enums
/// While that may work for structs, it will not work for enums.
/// Enums require a way to determine which variant is being used.
/// The standard solution I have come up with is to store
/// that as a [u8] placed before the data.
/// (If you need more than a [u8] to store the variant numbers,
/// then you could store it in a larger number but I don't think
/// that situation is likely)
/// This is most simply shown with enums that do not contain
/// data and just differentiate which variant is being used.
/// ```
/// # use abes_nice_things::FromBinary;
/// # use std::io::Read;
/// enum Example {
///     Variant1,
///     Variant2
/// }
/// impl FromBinary for Example {
///     fn from_binary(binary: &mut dyn Read) -> Self {
///         match u8::from_binary(binary) {
///             0 => Example::Variant1,
///             1 => Example::Variant2,
///             _ => unreachable!("Bad format")
///   // Notice this ^^^^^^^^^^^^^^^^^^^^^^^^^^
///         }
///     }
/// }
/// ```
/// Because [u8] implements [ToBinary] we can just specify
/// a [u8] to show the variants and then just convert it
/// [from binary](FromBinary::from_binary).
/// Also, because match requires you handle all possiblities,
/// we do need a default case, however, assuming the correct
/// binary is given in the correct format, it should never
/// be reached and is therefore [unreachable].
/// Notably, you don't actually have to conform to how I
/// show which variant is being used,
/// the only thing that matters is that it is able to
/// determine which is being used, and you do the opposite
/// in order to convert [to binary](ToBinary::to_binary).
/// 
/// While this works for some enums, it does not work if
/// they have data in their variants. The way we get around
/// that is by just by converting all their data [to binary](ToBinary::to_binary).
/// ```
/// # use abes_nice_things::FromBinary;
/// # use std::io::Read;
/// enum Example<T: FromBinary> {
///     EmptyVariant,
///     TupleVariant(u8, T),
///     StructVariant {
///         field1: String,
///         field2: Vec<T>
///     }
/// }
/// impl<T: FromBinary> FromBinary for Example<T> {
///     fn from_binary(binary: &mut dyn Read) -> Self {
///         match u8::from_binary(binary) {
///             0 => Example::EmptyVariant,
///             1 => Example::TupleVariant(
///                     u8::from_binary(binary),
///                     T::from_binary(binary)
///                 ),
///             2 => Example::StructVariant {
///                     field1: String::from_binary(binary),
///                     field2: Vec::from_binary(binary)
///                     // Notice lack of generics
///                 },
///             _ => unreachable!("Bad format")
///         }
///     }
/// }
/// ```
/// Because [u8] and [String] both unconditionally
/// implement [FromBinary], we can just get them
/// from the binary. Similarly, T and [Vec<T>]
/// both implement [FromBinary] so long as
/// T implements [FromBinary]. Meaning that
/// because we ensure that T must implement
/// [FromBinary], we can just do them similarly
/// to the [u8] and [String].
pub trait FromBinary {
    /// This method allows for easier converson
    /// from binary while staying safe
    /// (assuming you are converting what you think you are).
    /// 
    /// Specifically, it takes in something that implements [Read]
    /// and returns the data type you want.
    /// # Examples
    /// Reading from a file:
    /// ```no_run
    /// # use abes_nice_things::FromBinary;
    /// use std::io::Read;
    /// use std::fs::File;
    /// #[derive(Debug)]
    /// struct Data(i64, String);
    /// impl FromBinary for Data {
    ///     fn from_binary(binary: &mut dyn Read) -> Self {
    ///         Data(
    ///             i64::from_binary(binary),
    ///             String::from_binary(binary)
    ///         )
    ///     }
    /// }
    /// fn main() {
    ///     println!("{:?}", Data::from_binary(&mut File::open("source").unwrap()));
    /// }
    /// ```
    /// Reading from a [VecDeque](std::collections::VecDeque) using the same Data struct:
    /// ```no_run
    /// # use abes_nice_things::FromBinary;
    /// use std::io::Read;
    /// use std::collections::VecDeque;
    /// # #[derive(Debug)]
    /// # struct Data(i64, String);
    /// # impl FromBinary for Data {
    /// #     fn from_binary(binary: &mut dyn Read) -> Self {
    /// #         Data(
    /// #             i64::from_binary(binary),
    /// #             String::from_binary(binary)
    /// #         )
    /// #     }
    /// # }
    /// fn main() {
    ///     # #[cfg(not(any(debug_assertions, not(debug_assertions))))]
    ///     let mut binary = VecDeque::from(/*Some sort of binary data*/);
    ///     # let mut binary = VecDeque::new();
    ///     println!("{:?}", Data::from_binary(&mut binary));
    /// }
    /// ```
    /// Reading from a [TcpStream](std::net::TcpStream)
    /// ```no_run
    /// # use abes_nice_things::FromBinary;
    /// use std::io::Read;
    /// use std::net::TcpStream;
    /// # #[derive(Debug)]
    /// # struct Data(i64, String);
    /// # impl FromBinary for Data {
    /// #     fn from_binary(binary: &mut dyn Read) -> Self {
    /// #         Data (
    /// #             i64::from_binary(binary),
    /// #             String::from_binary(binary)
    /// #         )
    /// #     }
    /// # }
    /// fn main() {
    ///     let mut stream: TcpStream = {
    ///         // Something to create the TcpStream
    ///     # todo!()
    ///     };
    ///     println!("{:?}", Data::from_binary(&mut stream));
    /// }
    /// ```
    /// Notably, the same implentation of [FromBinary] was
    /// able to be used for all three examples!
    /// Because [from_binary](FromBinary::from_binary)
    /// takes in anything that implements [Read],
    /// So long as your data source implements [Read],
    /// no extra code has to be written.
    /// 
    /// For more infomation see the [trait docs](FromBinary)
    fn from_binary(binary: &mut dyn Read) -> Self;
}
/// This trait is designed to allow for easier conversion to binary
/// Long gone shall the days be of manually using [Write]
/// and filling from buffers. Now you can literally just
/// ```ignore
/// MyStruct::to_binary(/* wherever you are putting the data */)
/// ```
/// The secret to this working nicely is its modularity.
/// So long as all the data in a struct implements
/// [ToBinary], you can just call [to_binary](ToBinary::to_binary)
/// on its individual fields.
/// ```
/// # use abes_nice_things::ToBinary;
/// # use std::io::Write;
/// struct MyStruct {
///     field1: String,
///     field2: u8,
/// }
/// impl ToBinary for MyStruct {
///     fn to_binary(&self, binary: &mut dyn Write) {
///         self.field1.to_binary(binary);
///         self.field2.to_binary(binary);
///     }
/// }
/// ```
/// Both [String] and [u8] both implement [ToBinary],
/// meaning that we can just convert them to binary.
/// We don't have to do anything special and just
/// give them the binary without doing anything because
/// I built these around [Read] and [Write], which
/// will move the cursor as you read, meaning that it
/// moves to the data correctly, so long as you [Read]
/// and [Write] in the same order. Essentially, you
/// need to [write](Write::write) fields in the same
/// order you [read](Read::read) fields. In this case
/// I [write](Write::write) field1, then field2.
/// Because of this, I need to [read](Read::read) field1,
/// then field2. If I don't then it would [read](Read::read)
/// field1's bytes for field2 and vice versa, which would
/// immediately corrupt your data.
/// 
/// # Enums
/// While that may work for structs, enums are one
/// of multiple things and we need to identify which
/// variant is being used. I personally use [u8] to
/// indicate which variant, but you could use anything
/// so long as it is able to determine which
/// variant you are using and you are using it
/// in [FromBinary] to identify which variant
/// was stored.
/// ```
/// # use abes_nice_things::ToBinary;
/// # use std::io::Write;
/// enum Example {
///     Variant1,
///     Variant2
/// }
/// impl ToBinary for Example {
///     fn to_binary(&self, binary: &mut dyn Write) {
///         match self {
///             Example::Variant1 => 0_u8.to_binary(binary),
///             Example::Variant2 => 1_u8.to_binary(binary)
///                    // Notice this ^^^
///         }
///     }
/// }
/// ```
/// Much like before, because [u8] implements [ToBinary],
/// we can just convert it easily. Also, the underscore
/// format that I used is (from what I've found), the
/// easiest way to specify a number and its type.
/// Notably, the number that you use for each variant
/// is arbitrary, but must mirror what you use when converting
/// [FromBinary]. In this case, if I messed it up, it would
/// think that Variant1 is Variant2 and Variant2 is Variant1,
/// which would immediately corrupt your data.
/// 
/// More complicated example:
/// ```
/// # use abes_nice_things::ToBinary;
/// # use std::io::Write;
/// enum Example<T: ToBinary> {
///     EmptyVariant,
///     TupleVariant(u8, T),
///     StructVariant {
///         field1: String,
///         field2: Vec<T>
///     }
/// }
/// impl<T: ToBinary> ToBinary for Example<T> {
///     fn to_binary(&self, binary: &mut dyn Write) {
///         match self {
///             Example::EmptyVariant => 0_u8.to_binary(binary),
///             Example::TupleVariant (field1, field2) => {
///                 1_u8.to_binary(binary);
///                 field1.to_binary(binary);
///                 field2.to_binary(binary);
///             }
///             Example::StructVariant { field1, field2 } => {
///                 2_u8.to_binary(binary);
///                 field1.to_binary(binary);
///                 field2.to_binary(binary);
///             }
///         }
///     }
/// }
/// ```
/// When implementing this, you must ALWAYS
/// include something to indicate the variant being
/// used. Aside from that, it is like a combination of
/// how we did this for structs and how we did this for
/// simple enums. As such, it follows similar rules.
/// As said before, you need to show which variant is
/// being used, but you also need the data in the 
/// variants to be consistently ordered between the
/// [to](ToBinary::to_binary) operation and the
/// [from](FromBinary::from_binary) operation.
pub trait ToBinary {
    /// This method allows for easier conversion
    /// to binary cheaply and simply.
    /// 
    /// The basic usage is:
    /// ```ignore
    /// data.to_binary(/* Where you want the data to go */)
    /// ```
    /// For more specific examples:
    /// 
    /// To [File](std::fs::File):
    /// ```no_run
    /// # use abes_nice_things::ToBinary;
    /// # use std::io::Write;
    /// # struct Data(());
    /// # impl ToBinary for Data { fn to_binary(&self, binary: &mut dyn Write) {self.0.to_binary(binary)}}
    /// # fn main() {
    /// # let data = Data(());
    /// data.to_binary(
    ///     &mut std::fs::File::open("target_file").unwrap()
    /// );
    /// # }
    /// ```
    /// To [VecDeque](std::collections::VecDeque)
    /// ```
    /// # use abes_nice_things::ToBinary;
    /// # use std::collections::VecDeque;
    /// # fn main() {
    /// # let data = "Controlling robots is my game";
    /// let mut binary = VecDeque::new();
    /// data.to_binary(&mut binary);
    /// # }
    /// ```
    /// To [TcpStream](std::net::TcpStream)
    /// ```ignore
    /// data.to_binary(stream);
    /// ```
    /// As you can probably tell, anything that implements
    /// [Write] is a valid binary for this method.
    /// For more information/implementation instructions,
    /// look at [trait level docs](ToBinary)
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
#[cfg(not(feature = "dyn_binary"))]
impl FromBinary for usize {
    fn from_binary(binary: &mut dyn Read) -> Self {
        u32::from_binary(binary) as usize
    }
}
#[cfg(not(feature = "dyn_binary"))]
impl ToBinary for usize {
    fn to_binary(&self, binary: &mut dyn Write) {
        (*self as u32).to_binary(binary)
    }
}
#[cfg(not(feature = "dyn_binary"))]
impl FromBinary for isize {
    fn from_binary(binary: &mut dyn Read) -> Self {
        i32::from_binary(binary) as isize
    }
}
#[cfg(not(feature = "dyn_binary"))]
impl ToBinary for isize {
    fn to_binary(&self, binary: &mut dyn Write) {
        (*self as i32).to_binary(binary)
    }
}
macro_rules! vec_helper {
    () => {
        fn to_binary(&self, binary: &mut dyn Write) {
            self.len().to_binary(binary);
            for item in self.iter() {
                item.to_binary(binary)
            }
        }
    }
}
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
        let len = usize::from_binary(binary);
        let mut buf = vec![0; len];
        binary.read_exact(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }
}
impl ToBinary for String {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.len().to_binary(binary);
        binary.write_all(self.as_bytes()).unwrap();
    }
}
impl ToBinary for &str {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.to_string().to_binary(binary)
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
    vec_helper!();
}
impl<T: FromBinary> FromBinary for std::collections::VecDeque<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Vec::from_binary(binary).into()
    }
}
impl<T: ToBinary> ToBinary for std::collections::VecDeque<T> {
    // Can't convert to Vec to have it be the exact same thing
    // but using Vec format so that going from binary is easier
    vec_helper!();
}
impl<T: FromBinary> FromBinary for std::collections::LinkedList<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Vec::from_binary(binary).into_iter().collect()
    }
}
impl<T: ToBinary> ToBinary for std::collections::LinkedList<T> {
    // WHY
    // why does LinkedList break the standard of .into_iter() returning an iterator over non-references?
    // I get that no one cares about linked lists but still
    vec_helper!();
}
impl<T: FromBinary + std::hash::Hash + Eq, S: FromBinary + std::hash::BuildHasher> FromBinary for std::collections::HashSet<T, S> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        // Can't just use the Vec implementation for this
        // Format:
            // capacity: usize/u32
            // state
            // values
        let cap = usize::from_binary(binary);
        let mut out = Self::with_capacity_and_hasher(
            cap,
            S::from_binary(binary)
        );
        for _ in 0..cap {
            out.insert(T::from_binary(binary));
        }
        return out
    }
}
impl<T: ToBinary, S: ToBinary> ToBinary for std::collections::HashSet<T, S> {
    // Seriously? Again?
    // I'm noticing a pattern...
    fn to_binary(&self, binary: &mut dyn Write) {
        self.len().to_binary(binary);
        self.hasher().to_binary(binary);
        for item in self.iter() {
            item.to_binary(binary)
        }
    }
}
#[cfg(feature = "transmute_binary")]
impl FromBinary for std::hash::RandomState {
    fn from_binary(binary: &mut dyn Read) -> Self {
        unsafe {
            transmute::<RandomState, Self>(RandomState::from_binary(binary))
        }
    }
}
#[cfg(feature = "transmute_binary")]
impl ToBinary for std::hash::RandomState {
    fn to_binary(&self, binary: &mut dyn Write) {
        unsafe {
            transmute::<Self, RandomState>(self.clone()).to_binary(binary);
        }
    }
}
impl<K: FromBinary + Eq + std::hash::Hash, V: FromBinary, S: FromBinary + std::hash::BuildHasher> FromBinary for std::collections::HashMap<K, V, S> {
    // layout:
        // cap: usize/u32
        // hasher: S
        // data: [(K, V)]
    fn from_binary(binary: &mut dyn Read) -> Self {
        let cap = usize::from_binary(binary);
        let mut out = Self::with_capacity_and_hasher(
            cap,
            S::from_binary(binary)
        );
        for _ in 0..cap {
            out.insert(K::from_binary(binary), V::from_binary(binary));
        }
        return out
    }
}
impl<K: ToBinary, V: ToBinary, S: ToBinary> ToBinary for std::collections::HashMap<K, V, S> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.len().to_binary(binary);
        self.hasher().to_binary(binary);
        for (key, val) in self.iter() {
            key.to_binary(binary);
            val.to_binary(binary);
        }
    }
}
impl<T: FromBinary + Ord> FromBinary for std::collections::BinaryHeap<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Vec::from_binary(binary).into()
    }
}
impl<T: ToBinary> ToBinary for std::collections::BinaryHeap<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.as_slice().to_binary(binary);
    }
}
impl<T: ToBinary> ToBinary for &[T] {
    vec_helper!();
}
impl<T: FromBinary + Ord> FromBinary for std::collections::BTreeSet<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Vec::from_binary(binary).into_iter().collect()
    }
}
impl<T: ToBinary> ToBinary for std::collections::BTreeSet<T> {
    vec_helper!();
}
impl<K: FromBinary + Ord, V: FromBinary> FromBinary for std::collections::BTreeMap<K, V> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Vec::from_binary(binary).into_iter().collect()
    }
}
impl<K: ToBinary, V: ToBinary> ToBinary for std::collections::BTreeMap<K, V> {
    vec_helper!();
}
impl FromBinary for std::alloc::Layout {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::from_size_align(
            usize::from_binary(binary),
            usize::from_binary(binary)
        ).expect("Invalid binary")
    }
}
impl ToBinary for std::alloc::Layout {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.size().to_binary(binary);
        self.align().to_binary(binary);
    }
}
#[cfg(feature = "transmute_binary")]
impl FromBinary for std::alloc::LayoutError {
    // Contains no data so nothing is read
    fn from_binary(_binary: &mut dyn Read) -> Self {
        unsafe {
            transmute(())
        }
    }
}
impl ToBinary for std::alloc::LayoutError {
    // Contains no data so nothing is written
    fn to_binary(&self, _binary: &mut dyn Write) {}
}
#[cfg(feature = "transmute_binary")]
impl FromBinary for std::array::TryFromSliceError {
    fn from_binary(_binary: &mut dyn Read) -> Self {
        unsafe {
            transmute(())
        }
    }
}
impl ToBinary for std::array::TryFromSliceError {
    fn to_binary(&self, _binary: &mut dyn Write) {
        
    }
}
impl<T: FromBinary> FromBinary for Option<T> {
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
impl<T: ToBinary> ToBinary for Option<&T> {
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
impl<T: FromBinary, U: FromBinary> FromBinary for (T, U) {
    fn from_binary(binary: &mut dyn Read) -> Self {
        (
            T::from_binary(binary),
            U::from_binary(binary),
        )
    }
}
impl<T: ToBinary, U: ToBinary> ToBinary for (&T, &U) {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.0.to_binary(binary);
        self.1.to_binary(binary);
    }
}
impl<T: FromBinary, U: FromBinary, I: FromBinary> FromBinary for (T, U, I) {
    fn from_binary(binary: &mut dyn Read) -> Self {
        (
            T::from_binary(binary),
            U::from_binary(binary),
            I::from_binary(binary)
        )
    }
}
impl<T: ToBinary, U: ToBinary, I: ToBinary> ToBinary for (&T, &U, &I) {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.0.to_binary(binary);
        self.1.to_binary(binary);
        self.2.to_binary(binary);
    }
}
impl<T: FromBinary, U: FromBinary, I: FromBinary, O: FromBinary> FromBinary for (T, U, I, O) {
    fn from_binary(binary: &mut dyn Read) -> Self {
        (
            T::from_binary(binary),
            U::from_binary(binary),
            I::from_binary(binary),
            O::from_binary(binary)
        )
    }
}
impl<T: ToBinary, U: ToBinary, I: ToBinary, O: ToBinary> ToBinary for (&T, &U, &I, &O) {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.0.to_binary(binary);
        self.1.to_binary(binary);
        self.2.to_binary(binary);
        self.3.to_binary(binary);
    }
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
#[cfg(feature = "transmute_binary")]
impl FromBinary for std::time::Instant {
    fn from_binary(binary: &mut dyn Read) -> Self {
        transmute(Instant::from_binary(binary))
    }
}
#[cfg(feature = "transmute_binary")]
impl ToBinary for std::time::Instant {
    fn to_binary(&self, binary: &mut dyn Write) {
        transmute::<Self, Instant>(self).to_binary(binary)
    }
}
#[cfg(feature = "transmute_binary")]
impl FromBinary for std::time::SystemTime {
    fn from_binary(binary: &mut dyn Read) -> Self {
        transmute(Instant::from_binary(binary))
    }
}
#[cfg(feature = "transmute_binary")]
impl ToBinary for std::time::SystemTime {
    fn to_binary(&self, binary: &mut dyn Write) {
        transmute::<Self, Instant>(self).to_binary(binary)
    }
}
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
impl FromBinary for std::backtrace::BacktraceStatus {
    fn from_binary(binary: &mut dyn Read) -> Self {
        match u8::from_binary(binary) {
            0 => Self::Captured,
            1 => Self::Disabled,
            2 => Self::Unsupported,
            _ => unreachable!("Either you have a binary with an invalid format, or you need to update rust/libraries")
        }
    }
}
impl ToBinary for std::backtrace::BacktraceStatus {
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            Self::Captured => 0_u8.to_binary(binary),
            Self::Disabled => 1_u8.to_binary(binary),
            Self::Unsupported => 2_u8.to_binary(binary),
            _ => unreachable!("Put an issue at the repo for abes_nice_things by Course-Brains saying that BacktraceStatus has been modified and no longer works")
        }
    }
}
impl<T: FromBinary> FromBinary for std::cell::Cell<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::new(T::from_binary(binary))
    }
}
impl<T: ToBinary> ToBinary for std::cell::Cell<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        // It is safe actually because we never mutate the anything,
        // meaning that it is essentially the same as just &T
        unsafe {
            (&*self.as_ptr()).to_binary(binary);
        }
    }
}
impl<T: FromBinary> FromBinary for std::cell::OnceCell<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        match Option::<T>::from_binary(binary) {
            Some(data) => Self::from(data),
            None => Self::new()
        }
    }
}
impl<T: ToBinary> ToBinary for std::cell::OnceCell<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.get().to_binary(binary);
    }
}
impl<T: FromBinary> FromBinary for std::cell::UnsafeCell<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::new(T::from_binary(binary))
    }
}
impl<T: ToBinary> ToBinary for std::cell::UnsafeCell<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        unsafe {
            (&*self.get()).to_binary(binary)
        }
    }
}
impl FromBinary for std::ffi::CString {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::new(Vec::from_binary(binary)).expect("Bad format")
    }
}
impl ToBinary for std::ffi::CString {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.as_bytes().to_binary(binary);
    }
}
impl ToBinary for std::ffi::CStr {
    fn to_binary(&self, binary: &mut dyn Write) {
        std::ffi::CString::from(self).to_binary(binary)
    }
}
impl<T> FromBinary for std::marker::PhantomData<T> {
    fn from_binary(_binary: &mut dyn Read) -> Self {
        Self
    }
}
impl<T> ToBinary for std::marker::PhantomData<T> {
    fn to_binary(&self, _binary: &mut dyn Write) {}
}
impl FromBinary for std::marker::PhantomPinned {
    fn from_binary(_binary: &mut dyn Read) -> Self {
        Self
    }
}
impl ToBinary for std::marker::PhantomPinned {
    fn to_binary(&self, _binary: &mut dyn Write) {}
}
impl<T: FromBinary> FromBinary for std::mem::ManuallyDrop<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::new(T::from_binary(binary))
    }
}
impl<T: ToBinary> ToBinary for std::mem::ManuallyDrop<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.deref().to_binary(binary)
    }
}
impl<B: FromBinary, C: FromBinary> FromBinary for std::ops::ControlFlow<B, C> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        match u8::from_binary(binary) {
            0 => Self::Break(B::from_binary(binary)),
            1 => Self::Continue(C::from_binary(binary)),
            _ => unreachable!("Professor Bug, that is my name")
        }
    }
}
impl<B: ToBinary, C: ToBinary> ToBinary for std::ops::ControlFlow<B, C> {
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            Self::Break(data) => {
                0_u8.to_binary(binary);
                data.to_binary(binary);
            }
            Self::Continue(data) => {
                1_u8.to_binary(binary);
                data.to_binary(binary)
            }
        }
    }
}
macro_rules! non_zero_num_helper {
    ($($type: ty, $sub_type: ty)*) => {
        $(
            impl FromBinary for $type {
                fn from_binary(binary: &mut dyn Read) -> Self {
                    Self::new(<$sub_type>::from_binary(binary)).unwrap()
                }
            }
            impl ToBinary for $type {
                fn to_binary(&self, binary: &mut dyn Write) {
                    self.get().to_binary(binary)
                }
            }
        )*
    }
}
non_zero_num_helper!(
    std::num::NonZeroI8, i8
    std::num::NonZeroI16, i16
    std::num::NonZeroI32, i32
    std::num::NonZeroI64, i64
    std::num::NonZeroI128, i128
    std::num::NonZeroU8, u8
    std::num::NonZeroU16, u16
    std::num::NonZeroU32, u32
    std::num::NonZeroU64, u64
    std::num::NonZeroU128, u128
    std::num::NonZeroUsize, usize
    std::num::NonZeroIsize, isize
);
impl FromBinary for std::process::ExitCode {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::from(u8::from_binary(binary))
    }
}
#[cfg(feature = "transmute_binary")]
impl ToBinary for std::process::ExitCode {
    fn to_binary(&self, binary: &mut dyn Write) {
        transmute::<Self, u8>(self).to_binary(binary);
    }
}
#[cfg(feature = "transmute_binary")]
impl FromBinary for std::process::ExitStatus {
    fn from_binary(binary: &mut dyn Read) -> Self {
        transmute(i32::from_binary(binary))
    }
}
#[cfg(feature = "transmute_binary")]
impl ToBinary for std::process::ExitStatus {
    fn to_binary(&self, binary: &mut dyn Write) {
        transmute::<Self, i32>(self).to_binary(binary)
    }
}
impl<T: FromBinary> FromBinary for std::sync::Mutex<T> {
    fn from_binary(binary: &mut dyn Read) -> Self {
        Self::new(T::from_binary(binary))
    }
}
impl<T: ToBinary> ToBinary for std::sync::Mutex<T> {
    fn to_binary(&self, binary: &mut dyn Write) {
        self.lock().unwrap().to_binary(binary)
    }
}
macro_rules! atomic_helper {
    ($($type:ty, $sub_type: ty)*) => {
        $(
            impl FromBinary for $type {
                fn from_binary(binary: &mut dyn Read) -> Self {
                    Self::new(<$sub_type>::from_binary(binary))
                }
            }
            impl ToBinary for $type {
                fn to_binary(&self, binary: &mut dyn Write) {
                    self.load(std::sync::atomic::Ordering::Acquire).to_binary(binary)
                }
            }
        )*
    }
}
atomic_helper!(
    std::sync::atomic::AtomicBool, bool
    std::sync::atomic::AtomicU8, u8
    std::sync::atomic::AtomicU16, u16
    std::sync::atomic::AtomicU32, u32
    std::sync::atomic::AtomicU64, u64
    // Can't do 128 because it is unstable
    //std::sync::atomic::AtomicU128, u128
    std::sync::atomic::AtomicUsize, usize
    std::sync::atomic::AtomicI8, i8
    std::sync::atomic::AtomicI16, i16
    std::sync::atomic::AtomicI32, i32
    std::sync::atomic::AtomicI64, i64
    //std::sync::atomic::AtomicI128, i128
    std::sync::atomic::AtomicIsize, isize
);
impl FromBinary for std::sync::atomic::Ordering {
    fn from_binary(binary: &mut dyn Read) -> Self {
        match u8::from_binary(binary) {
            0 => Self::AcqRel,
            1 => Self::Acquire,
            2 => Self::Relaxed,
            3 => Self::Release,
            4 => Self::SeqCst,
            _ => unreachable!()
        }
    }
}
impl ToBinary for std::sync::atomic::Ordering {
    fn to_binary(&self, binary: &mut dyn Write) {
        match self {
            Self::AcqRel => 0_u8.to_binary(binary),
            Self::Acquire => 1_u8.to_binary(binary),
            Self::Relaxed => 2_u8.to_binary(binary),
            Self::Release => 3_u8.to_binary(binary),
            Self::SeqCst => 4_u8.to_binary(binary),
            _ => unreachable!("Tell Course-Brains that they changed atomic::Ordering")
        }
    }
}
#[cfg(feature = "transmute_binary")]
mod transmuters {
    use super::{FromBinary, ToBinary};
    pub struct RandomState {
        k0: u64,
        k1: u64
    }
    impl FromBinary for RandomState {
        fn from_binary(binary: &mut dyn std::io::Read) -> Self {
            Self {
                k0: u64::from_binary(binary),
                k1: u64::from_binary(binary)
            }
        }
    }
    impl ToBinary for RandomState {
        fn to_binary(&self, binary: &mut dyn std::io::Write) {
            self.k0.to_binary(binary);
            self.k1.to_binary(binary);
        }
    }
    pub struct Instant {
        tv_sec: u64,
        tv_nsec: u32
    }
    impl FromBinary for Instant {
        fn from_binary(binary: &mut dyn std::io::Read) -> Self {
            Self {
                tv_sec: u64::from_binary(binary),
                tv_nsec: u32::from_binary(binary)
            }
        }
    }
    impl ToBinary for Instant {
        fn to_binary(&self, binary: &mut dyn std::io::Write) {
            self.tv_sec.to_binary(binary);
            self.tv_nsec.to_binary(binary);
        }
    }
}
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
    mod vec_deque {
        use super::{FromBinary, ToBinary, VecDeque};
        #[test]
        fn simple() {
            let mut binary = VecDeque::new();
            let value = VecDeque::from([1,2,7,83]);
            value.to_binary(&mut binary);
            assert_eq!(value, VecDeque::from_binary(&mut binary))
        }
    }
}