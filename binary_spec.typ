#set heading(numbering: "1.a:")
#set page(numbering: "1")
#show link: underline
#show link: set text(blue)
#show text: set text(size: 14pt)
#title[Storage format of converted binary data used by ToBinary and FromBinary]
Abraham Sharnoff\
Document last updated on #datetime.today().display()
#outline()
#pagebreak(weak: true)
#heading(numbering: none)[Prelude]
If I say that something is stored as a type T, then look at
that type's section to figure out how it is represented in
binary.

If I say that something is stored as [T] then it is being
stored as a slice of T, meaning that it is consecutive T
bytes without any gaps and follows the same definition rules
as for storing T.

If I say that something is stored as [T; N] then it is being
stored as an array of T with length N meaning that it is
equivalent to a slice but has a consistent and known length.

If I am going to describe how to store an enum then it will
be using the following syntax:
- \[variant\]: \[format\]
- \[variant\]: \[format\]
- \[variant\]: \[format\]
Know that only one of those paths will be what gets saved.

If I am feeling particularly lazy and need to represent how
a struct gets saved then I may use a table as such:
#table(
  [\[field\]: \[format\]],
  [\[field\]: \[format\]],
  [\[field\]: \[format\]],
)
Know that ALL of the fields will be saved. They will be
saved in that specific order.

= Number Primitives <number>
#let u8 = link(<number>)[u8]
#let u16 = link(<number>)[u16]
#let u32 = link(<number>)[u32]
#let u64 = link(<number>)[u64]
#let u128 = link(<number>)[u128]
#let usize = link(<number>)[usize]
#let isize = link(<number>)[isize]
#let f32 = link(<number>)[f32]
All numbers are stored as [#u8\; N] where N is the number of
bytes in that number primitive. So a #u8 will have an N of
1 and a #u16 will have an N of 2.

The first weird part is with floats because they are
technically converted to the unsigned integer of the same
before writing, so an #f32 gets turned into a #u32 first.
But this doesn't actually matter because it leaves the bytes
untouched in little endian order.

Which brings be to weird part number 2, all integers are
stored in little endian byte order, which usually means no
operations are done because computers generally store them
in little endian byte order but it is good to enforce it
just in case.

All of the NonZero number variants are stored as their
zeroable counterparts.

All of the Atomic number variants are stored as there normal
counterparts and when writing are read with Acquire
ordering.
== Important distinction about #usize and #isize
#usize and #isize are stored as though they were their 64
bit counterparts by default but if the "dyn_binary" feature
flag is set then they are instead stored in their native
size.
= char <char>
#let char = link(<char>)[char]
#{char}s are stored as #u32, don't ask me why. That's just
what I can convert them to and from.
= bool <bool>
#let bool = link(<bool>)[bool]
- false: 0\_#u8
- true: 1\_#u8
= String <String>
#let String = link(<String>)[String]
#table(
  [len: #usize],
  [bytes of data: \[#u8\]]
)
= &str
It only implements ToBinary and does so in the same way as #String\.
= Vec\<T\> <Vec>
#let Vec = link(<Vec>)[Vec]
#table(
  [len: #usize],
  [contents: \[T\]]
)
Any time something says that it is stored the same as a
#Vec, gets converted to and from a #Vec before and after
being in binary, usually through a to_vec method.
= VecDeque\<T\>
Same as #Vec
= LinkedList\<T\>
Same as #Vec
= HashSet\<T\>
== With base hasher
Same as #Vec
== With non base hasher S
#table(
  [len: #usize],
  [hasher: S],
  [contents: \[T\]]
)
The way we are getting the contents is iterating, not
getting the raw bytes.
= HashMap\<K, V\>
== With base hasher
Same as a #Vec of (K, V)
== With non base hasher S
#table(
  [len; #usize],
  [hasher: S],
  [contents: \[(K, V)\]]
)
= BinaryHeap\<T\>
Same as #Vec
= BTreeSet\<T\>
Same as #Vec
= BTreeMap\<K, V\>
Same as #Vec of (K, V)
= std::alloc::Layout
#table(
  [size: #usize],
  [align: #usize]
)
= Option\<T\> <Option>
#let Option = link(<Option>)[Option]
- Some(T): (true, T)
- None: false
= Result\<T, E\>
- Ok(T): (false, T)
- Err(E): (true, E)
= ()
No op
= (T, U)
#table(
  [T],
  [U]
)
= (T, U, I)
#table(
  [T],
  [U],
  [I]
)
= (T, U, I, O)
#table(
  [T],
  [U],
  [I],
  [O]
)
= std::cmp::Ordering
- Equal: 0
- Less: 1
- Greater: 2
All as #u8
= Box\<T\>
Same as T
= Ipv4Addr <Ipv4Addr>
#let Ipv4Addr = link(<Ipv4Addr>)[Ipv4Addr]
Stored as #u32
= Ipv6Addr <Ipv6Addr>
#let Ipv6Addr = link(<Ipv6Addr>)[Ipv6Addr]
Stored as #u128
= IpAddr
- V4: (false, #Ipv4Addr)
- V6: (true, #Ipv6Addr)
= SocketAddrV4 <SocketAddrV4>
#let SocketAddrV4 = link(<SocketAddrV4>)[SocketAddrV4]
#table(
  [ip: #Ipv4Addr],
  [port: #u16]
)
= SocketAddrV6 <SocketAddrV6>
#let SocketAddrV6 = link(<SocketAddrV6>)[SocketAddrV6]
#table(
  [ip: #Ipv6Addr],
  [port: #u16],
  [flowinfo: #u32],
  [scope_id: #u32]
)
= SocketAddr
- V4: (false, #SocketAddrV4)
- V6: (true, #SocketAddrV6)
= std::io::ErrorKind
I'm not typing all of that again, go to from_binary.rs and
search for "ErrorKind" if you want to know how it works.
= std::ops::Bound\<T\>
- Excluded(point: T): (0, T),
- Included(point: T): (1, T),
- Unbounded: 2
Discriminant is #u8
= std::ops::Range\<T\>
#table(
  [start: T],
  [end: T]
)
= std::ops::RangeFrom\<T\>
Stored as T
= std::ops::RangeInclusive\<T\>
#table(
  [start: T],
  [end: T]
)
= std::ops::RangeTo\<T\>
Stored as T
= std::ops::RangeToInclusive\<T\>
Stored as T
= Duration <Duration>
#let Duration = link(<Duration>)[Duration]
#table(
  [secs: #u64],
  [subsec_nanos: #u32]
)
= SystemTime
Stored as #Duration since SystemTime::UNIX_EPOCH
= BacktraceStatus <BacktraceStatus>
- Captured: 0
- Disabled: 1
- Unsupported: 2
Discriminant is #u8.

If your #link(<BacktraceStatus>)[BacktraceStatus] has more
variants then this code is no longer fully valid.
= Cell\<T\>
Stored as T
= OnceCell\<T\>
Stored as #Option\<T\>
= UnsafeCell\<T\>
Stored as T
= CString
Stored as #Vec\<T\>
= PhantomData\<T\>
No op
= PhantomPinned\<T\>
No op
= ManuallyDrop\<T\>
Same as T
= std::ops::ControlFlow\<B, C\>
- Break(value: B): (false, value),
- Continue(value: C): (true, value)
= std::process::ExitCode
Stored as #u8
= Mutex\<T\>
Stored as \<T\>. Does acquire lock in order to write and
will return Err if poisoned.
= std::sync::atomic::Ordering
- AcqRel: 0
- Acquire: 1
- Relaxed: 2
- Release: 3
- SeqCst: 4
Discriminant is #u8