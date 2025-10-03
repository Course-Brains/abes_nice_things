use crate::{FromBinary, ToBinary};
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicUsize;
static SPLITS: AtomicUsize = AtomicUsize::new(1);
type Halves<T> = (ReadHalf<T>, WriteHalf<T>);
/// This is a trait designed to [split](Split::split) things that
/// implement both [Read](std::io::Read) and [Write](std::io::Write)
/// into two separate things which each implement one of them.
/// For an example of the benefits, let's look at [TcpStream](std::net::TcpStream),
/// with [TcpStream](std::net::TcpStream) it can be useful to have it be written to
/// and read from in two different places. Under normal circumstances, the only way
/// to do that would be to clone it like this:
/// ```ignore
/// let (read, write) = (stream.try_clone().unwrap(), stream);
/// ```
/// However, this has a glaring issue. There is no guarantee that the people using
/// read and write won't missuse it. For example, someone who has the write half
/// may need to read something from it, and think "oh, it's just one thing, it will
/// be fine and won't affect anything" however, them taking that action removes the
/// guarantee that the read half is reading what it thinks it is, I don't like that.
/// As such, I made some types to enforce it. With those being [ReadHalf] and
/// [WriteHalf], which simply enforce only [writing](std::io::Write::write) to [WriteHalf]
/// and only [reading](std::io::Read::read) to [ReadHalf].
///
/// In relation to those, [Split](Split::split) simply creates them. And
/// [recombine](Split::recombine) consumes both halves to get the original value.

pub trait Split: std::io::Read + std::io::Write
where
    Self: Sized,
{
    /// This splits something which implements both [Read](std::io::Read) and
    /// [Write](std::io::Write) into two different halves such that one will
    /// [write](std::io::Write::write) in place of the original, and the
    /// other will [read](std::io::Read::read) in place of the original.
    /// For example:
    /// ```ignore
    /// let (read_half, write_half) = tcp_stream.split().unwrap();
    /// ```
    /// Will create the two distinct halves to the two variables named
    /// after them.
    ///
    /// The benefit of doing this is that this guarantees that there will
    /// only be one [Writer](std::io::Write) and one [Reader](std::io::Read).
    /// As opposed to simply cloning the original, which would allow both
    /// to [read](std::io::Read::read) and [write](std::io::Write::write).
    fn split(self) -> Result<Halves<Self>, std::io::Error>;
    /// This consumes the two halves of the *same* source and if they were
    /// from the same source, will return the original, otherwise, it will
    /// return None. For more information on how to
    /// create those halves, see [split](Split::split).
    ///
    /// For example:
    /// ```no_run
    /// # use abes_nice_things::Split;
    /// # use std::fs::File;
    /// # fn main() {
    /// let original = File::open("~/.vimrc").unwrap();
    /// // split consumes the original
    /// let (read, write) = original.split().unwrap();
    /// // recombine consumes the splits
    /// let recombined = File::recombine(read, write).unwrap();
    /// # }
    /// ```
    /// However, that only works if you are recombining splits from the same
    /// source, if they are different, [recombine](Split::recombine) will
    /// return [None].
    /// ```no_run
    /// # use abes_nice_things::Split;
    /// # use std::fs::File;
    /// # fn main() {
    /// let original1 = File::open("~/.vimrc").unwrap();
    /// let original2 = File::open("~/.cargo/bin/rustup").unwrap();
    /// let (read1, _) = original1.split().unwrap();
    /// let (_, write2) = original2.split().unwrap();
    /// // invalid recombine
    /// assert!(File::recombine(read1, write2).is_none());
    /// # }
    /// ```
    /// Due to the read and write halves being from different sources,
    /// they cannot be recombined.
    ///
    /// In order to check if two halves are from the same source, they
    /// implement [PartialEq] and can be compared to one another:
    /// ```no_run
    /// # use abes_nice_things::Split;
    /// # use std::fs::File;
    /// # fn main() {
    /// let original1 = File::open("~/.vimrc").unwrap();
    /// let original2 = File::open("~/.cargo/bin/rustup").unwrap();
    /// let (read1, write1) = original1.split().unwrap();
    /// let (read2, write2) = original2.split().unwrap();
    /// assert!(read1 == write1);
    /// assert!(read2 == write2);
    /// assert!(read1 != write2);
    /// assert!(read2 != write1);
    /// # }
    /// ```
    /// However, while [ReadHalf] can be compared to [WriteHalf] and
    /// vice versa, comparing a [ReadHalf] and another [ReadHalf] (same
    /// for [WriteHalf], just not mentioning it), it will not check if
    /// they are from the same source, and will instead compare the
    /// [Reader](std::io::Read) or [Writer](std::io::Write) inside while
    /// ignoring the id because there is no situation where an two
    /// [ReadHalf] will have the same id(same for [WriteHalf])
    fn recombine(read: ReadHalf<Self>, write: WriteHalf<Self>) -> Option<Self> {
        match read.1 == write.1 {
            // They are from the same source, yay!
            true => Some(read.0),
            // They are not from the same source, booooooooo
            false => None,
        }
    }
}
impl Split for std::net::TcpStream {
    fn split(self) -> Result<(ReadHalf<Self>, WriteHalf<Self>), std::io::Error> {
        let id =
            NonZeroUsize::new(SPLITS.fetch_add(1, std::sync::atomic::Ordering::SeqCst)).unwrap();
        Ok((
            ReadHalf(self.try_clone()?, Some(id)),
            WriteHalf(self, Some(id)),
        ))
    }
}
impl Split for std::fs::File {
    fn split(self) -> Result<(ReadHalf<Self>, WriteHalf<Self>), std::io::Error> {
        // Intentionally not accounting for Seek because
        // it is not independant from Reading and Writing
        let id =
            NonZeroUsize::new(SPLITS.fetch_add(1, std::sync::atomic::Ordering::SeqCst)).unwrap();
        Ok((
            ReadHalf(self.try_clone()?, Some(id)),
            WriteHalf(self, Some(id)),
        ))
    }
}
/// This holds anything that implements [Write](std::io::Write)
/// and limits that to be the only way you can interact with it.
/// This is useful for limiting things that implement both
/// [Read](std::io::Read) and [Write](std::io::Write) such that
/// it cannot be [read](std::io::Read::read) from.
///
/// Most of the time, this is created by calling [split](Split::split)
/// however it can be created manually.
/// ```
/// # use abes_nice_things::split::WriteHalf;
/// # use std::collections::VecDeque;
/// # fn main() {
/// let vec_deque = VecDeque::new();
/// // ^ can be written to or read from
/// let write_half = WriteHalf::new(vec_deque);
/// // ^ can only be written to
/// # }
/// ```
/// Most of the time, this should not be created manually,
/// but the option is there.
///
/// Notably, this also stores an id which is used to tell if
/// a [WriteHalf] and [ReadHalf] came from the same source
/// and can therefore be merged in [recombine](Split::recombine).
///
/// For more information on how this
/// is usually created, see [Split]
#[derive(Debug)]
pub struct WriteHalf<W: std::io::Write>(W, Option<NonZeroUsize>);
impl<W: std::io::Write> WriteHalf<W> {
    /// This creates an instance of [WriteHalf] containing
    /// the provided [Writer](std::io::Write).
    /// ```
    /// # use abes_nice_things::split::WriteHalf;
    /// # fn main() {
    /// let writer = Vec::new();
    /// let write_half = WriteHalf::new(writer);
    /// # }
    /// ```
    /// Notably, this does not have its id set, however,
    /// if you do need it set for some reason(I wouldn't
    /// recommend that) you can by using
    /// [new_id](WriteHalf::new_id)
    pub const fn new(write: W) -> WriteHalf<W> {
        WriteHalf(write, None)
    }
    /// This creates an instance of [WriteHalf] with
    /// the id set. I can't think of a use case where
    /// you would need this, but in case you do, this
    /// is here. Always remember though, this is unsafe
    /// for a reason. The id is used to determine if
    /// a [ReadHalf] and [WriteHalf] are from the same
    /// source for merging purposes. But they don't
    /// actually get merged, it just takes what was in
    /// the [ReadHalf] and returns that. Meaning that
    /// unless they were from the same source, the
    /// magic trick stops working and could leave
    /// people confused. So, it uses a unique id system
    /// to ensure that two halves from different sources
    /// can NEVER merge.
    ///
    /// This will set the id, and
    /// therefore allow merging so long as both halves
    /// contain the same type. Don't use this, it is a
    /// bad idea, unless you are implementing [Split]
    /// for something. In which case, go right ahead
    /// because this is the only way for you to do that.
    pub const unsafe fn new_id(write: W, id: NonZeroUsize) -> WriteHalf<W> {
        WriteHalf(write, Some(id))
    }
    /// This gets the [Writer](std::io::Write) stored
    /// in this, on the condition that it is not
    /// bound. [WriteHalf]s created through
    /// [Split] are bound to a [ReadHalf] and cannot
    /// be gotten through this, causing this to
    /// return [None]. However, if instead, it was
    /// created by [new](WriteHalf::new), it will
    /// return the contained [Writer](std::io::Write)
    /// ```
    /// # use abes_nice_things::split::WriteHalf;
    /// # fn main() {
    /// let writer = Vec::new();
    /// let write_half = WriteHalf::new(writer);
    /// let same_writer = write_half.get().unwrap();
    /// # }
    /// ```
    /// In this example, the original writer and the
    /// extracted writer are one and the same.
    /// However, if you tried to do this with a
    /// [WriteHalf] created through [Split], then it
    /// would panic.
    pub fn get(self) -> Option<W> {
        if self.1.is_none() {
            return Some(self.0);
        }
        None
    }
    /// Similar to [get](WriteHalf::get) except that
    /// it does not check if this is valid to get.
    /// Notably, it is NOT undefined behavior to call
    /// this, but it does destroy the guarantee given
    /// by this and [ReadHalf] in that there is only
    /// at most 1 [Writer](std::io::Write) and 1
    /// [Reader](std::io::Read) because you could
    /// use the extracted value to [Read](std::io::Read)
    /// when you should not be able to.
    pub unsafe fn get_unchecked(self) -> W {
        self.0
    }
    /// This gets the stored id of this instance. The
    /// id is used to check if a [ReadHalf] and
    /// [WriteHalf] came from the same source and can
    /// therefore be merged together to get the source
    /// back. Realistically, unless you are implementing
    /// [Split], you shouldn't be using this. If the
    /// instance was made manually through
    /// [new](WriteHalf::new), then this will return
    /// [None] instead of the id.
    pub const fn get_id(&self) -> Option<NonZeroUsize> {
        self.1
    }
}
impl<W: std::io::Write + PartialEq> PartialEq for WriteHalf<W> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}
impl<W: std::io::Write, R: std::io::Read> PartialEq<ReadHalf<R>> for WriteHalf<W> {
    fn eq(&self, other: &ReadHalf<R>) -> bool {
        self.1 == other.1
    }
    fn ne(&self, other: &ReadHalf<R>) -> bool {
        self.1 != other.1
    }
}
impl<W: std::io::Write> std::io::Write for WriteHalf<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.0.write_all(buf)
    }
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.0.write_fmt(fmt)
    }
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.0.write_vectored(bufs)
    }
}
impl<W: std::io::Write + FromBinary> FromBinary for WriteHalf<W> {
    fn from_binary(binary: &mut dyn std::io::Read) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(WriteHalf(
            W::from_binary(binary)?,
            Option::from_binary(binary)?,
        ))
    }
}
impl<W: std::io::Write + ToBinary> ToBinary for WriteHalf<W> {
    fn to_binary(&self, binary: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        self.0.to_binary(binary)?;
        self.1.as_ref().to_binary(binary)
    }
}
/// This holds anything that implements [Read](std::io::Read)
/// and limits that to be the only way you can interact with it.
/// This is useful for limiting things that implement both
/// [Read](std::io::Read) and [Write](std::io::Write) such that
/// it cannot be [written](std::io::Write::write) to.
///
/// Most of the time, this is created by calling [split](Split::split)
/// however it can be created manually.
/// ```
/// # use abes_nice_things::split::ReadHalf;
/// # use std::collections::VecDeque;
/// # fn main() {
/// let vec_deque = VecDeque::new();
/// // ^ can be written to or read from
/// let read_half = ReadHalf::new(vec_deque);
/// // ^ can only be read from
/// # }
/// ```
/// Most of the time, this should not be created manually,
/// but the option is there.
///
/// Notably, this also stores an id which is used to tell if
/// a [WriteHalf] and [ReadHalf] came from the same source
/// and can therefore be merged in [recombine](Split::recombine).
///
/// For more information on how this
/// is usually created, see [Split]
#[derive(Debug)]
pub struct ReadHalf<R: std::io::Read>(R, Option<NonZeroUsize>);
impl<R: std::io::Read> ReadHalf<R> {
    /// This creates an instance of [ReadHalf] containing
    /// the provided [Reader](std::io::Read).
    /// ```
    /// # use abes_nice_things::split::ReadHalf;
    /// # use std::collections::VecDeque;
    /// # fn main() {
    /// let reader = VecDeque::new();
    /// let read_half = ReadHalf::new(reader);
    /// # }
    /// ```
    /// Notably, this does not have its id set, however,
    /// if you do need it set for some reason(I wouldn't
    /// recommend that) you can by using
    /// [new_id](ReadHalf::new_id)
    pub const fn new(read: R) -> ReadHalf<R> {
        ReadHalf(read, None)
    }
    /// This creates an instance of [ReadHalf] with
    /// the id set. I can't think of a use case where
    /// you would need this, but in case you do, this
    /// is here. Always remember though, this is unsafe
    /// for a reason. The id is used to determine if
    /// a [ReadHalf] and [WriteHalf] are from the same
    /// source for merging purposes. But they don't
    /// actually get merged, it just takes what was in
    /// the [ReadHalf] and returns that. Meaning that
    /// unless they were from the same source, the
    /// magic trick stops working and could leave
    /// people confused. So, it uses a unique id system
    /// to ensure that two halves from different sources
    /// can NEVER merge.
    ///
    /// This will set the id, and
    /// therefore allow merging so long as both halves
    /// contain the same type. Don't use this, it is a
    /// bad idea, unless you are implementing [Split]
    /// for something. In which case, go right ahead
    /// because this is the only way for you to do that.
    pub const unsafe fn new_id(read: R, id: NonZeroUsize) -> ReadHalf<R> {
        ReadHalf(read, Some(id))
    }
    /// This gets the [Reader](std::io::Read) stored
    /// in this, on the condition that it is not
    /// bound. [ReadHalf]s created through
    /// [Split] are bound to a [WriteHalf] and cannot
    /// be gotten through this, causing this to
    /// return [None]. However, if instead, it was
    /// created by [new](ReadHalf::new), it will
    /// return the contained [Reader](std::io::Read)
    /// ```
    /// # use abes_nice_things::split::ReadHalf;
    /// # use std::collections::VecDeque;
    /// # fn main() {
    /// let reader = VecDeque::new();
    /// let read_half = ReadHalf::new(reader);
    /// let same_reader = read_half.get().unwrap();
    /// # }
    /// ```
    /// In this example, the original reader and the
    /// extracted reader are one and the same.
    /// However, if you tried to do this with a
    /// [ReadHalf] created through [Split], then it
    /// would panic.
    pub fn get(self) -> Option<R> {
        if self.1.is_none() {
            return Some(self.0);
        }
        None
    }
    /// Similar to [get](ReadHalf::get) except that
    /// it does not check if this is valid to get.
    /// Notably, it is NOT undefined behavior to call
    /// this, but it does destroy the guarantee given
    /// by this and [WriteHalf] in that there is only
    /// at most 1 [Writer](std::io::Write) and 1
    /// [Reader](std::io::Read) because you could
    /// use the extracted value to [Write](std::io::Write)
    /// when you should not be able to.
    pub unsafe fn get_unchecked(self) -> R {
        self.0
    }
    /// This gets the stored id of this instance. The
    /// id is used to check if a [ReadHalf] and
    /// [WriteHalf] came from the same source and can
    /// therefore be merged together to get the source
    /// back. Realistically, unless you are implementing
    /// [Split], you shouldn't be using this. If the
    /// instance was made manually through
    /// [new](WriteHalf::new), then this will return
    /// [None] instead of the id.
    pub const fn get_id(&self) -> Option<NonZeroUsize> {
        self.1
    }
}
impl<R: std::io::Read + PartialEq> PartialEq for ReadHalf<R> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0
    }
}
impl<W: std::io::Write, R: std::io::Read> PartialEq<WriteHalf<W>> for ReadHalf<R> {
    fn eq(&self, other: &WriteHalf<W>) -> bool {
        self.1 == other.1
    }
    fn ne(&self, other: &WriteHalf<W>) -> bool {
        self.1 != other.1
    }
}
impl<R: std::io::Read> std::io::Read for ReadHalf<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.0.read_exact(buf)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        self.0.read_to_end(buf)
    }
    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.0.read_vectored(bufs)
    }
}
impl<R: std::io::Read + FromBinary> FromBinary for ReadHalf<R> {
    fn from_binary(binary: &mut dyn std::io::Read) -> Result<Self, std::io::Error>
    where
        Self: Sized,
    {
        Ok(ReadHalf(
            R::from_binary(binary)?,
            Option::from_binary(binary)?,
        ))
    }
}
impl<R: std::io::Read + ToBinary> ToBinary for ReadHalf<R> {
    fn to_binary(&self, binary: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        self.0.to_binary(binary)?;
        self.1.as_ref().to_binary(binary)
    }
}
