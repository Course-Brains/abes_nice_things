use crate::{FromBinary, ToBinary};
static SPLITS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
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

pub trait Split: std::io::Read + std::io::Write where Self: Sized {
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
    /// return the two halves given to it. For more information on how to
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
    /// return [Err].
    /// ```no_run
    /// # use abes_nice_things::Split;
    /// # use std::fs::File;
    /// # fn main() {
    /// let original1 = File::open("~/.vimrc").unwrap();
    /// let original2 = File::open("~/.cargo/bin/rustup").unwrap();
    /// let (read1, _) = original1.split().unwrap();
    /// let (_, write2) = original2.split().unwrap();
    /// // invalid recombine
    /// File::recombine(read1, write2).unwrap_err();
    /// # }
    /// ```
    /// Due to the read and write halves being from different sources,
    /// they cannot be recombined.
    fn recombine(read: ReadHalf<Self>, write: WriteHalf<Self>) -> Result<Self, Halves<Self>> {
        match read.1 == write.1 {
            // They are from the same source, yay!
            true => Ok(read.0),
            // They are not from the same source, booooooooo
            false => Err((read, write))
        }
    }
}
impl Split for std::net::TcpStream {
    fn split(self) -> Result<(ReadHalf<Self>, WriteHalf<Self>), std::io::Error> {
        let id = SPLITS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok((
            ReadHalf(self.try_clone()?, Some(id)),
            WriteHalf(self, Some(id))
        ))
    }
}
impl Split for std::fs::File {
    fn split(self) -> Result<(ReadHalf<Self>, WriteHalf<Self>), std::io::Error> {
        // Intentionally not accounting for Seek because
        // it is not independant from Reading and Writing
        let id = SPLITS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok((
            ReadHalf(self.try_clone()?, Some(id)),
            WriteHalf(self, Some(id))
        ))
    }
}
/// This is a wrapper which restricts a type to only being able to write.
/// For instance, if it was wrapping a [File](std::fs::File), it would
/// no longer be able to [Read](std::io::Read).
///
/// This, along with its sister type [ReadHalf], are created by calling
/// [split](Split::split) on certain types, such as [File](std::fs::File)
/// and [TcpStream](std::net::TcpStream). While I'm not sure why you
/// would split a [File](std::fs::File), you can.
#[derive(Debug)]
pub struct WriteHalf<W: std::io::Write>(W, Option<usize>);
impl<W: std::io::Write> WriteHalf<W> {
     
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
    fn from_binary(binary: &mut dyn std::io::Read) -> Result<Self, std::io::Error> where Self: Sized {
        Ok(WriteHalf(
            W::from_binary(binary)?,
            Option::from_binary(binary)?
        ))
    }
}
impl<W: std::io::Write + ToBinary> ToBinary for WriteHalf<W> {
    fn to_binary(&self, binary: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        self.0.to_binary(binary)?;
        self.1.as_ref().to_binary(binary)
    }
}
#[derive(Debug)]
pub struct ReadHalf<R: std::io::Read>(R, Option<usize>);
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
    fn from_binary(binary: &mut dyn std::io::Read) -> Result<Self, std::io::Error> where Self: Sized {
        Ok(ReadHalf(
            R::from_binary(binary)?,
            Option::from_binary(binary)?
        ))
    }
}
impl<R: std::io::Read + ToBinary> ToBinary for ReadHalf<R> {
    fn to_binary(&self, binary: &mut dyn std::io::Write) -> Result<(), std::io::Error> {
        self.0.to_binary(binary)?;
        self.1.as_ref().to_binary(binary)
    }
}

