/// A trait to convert things inside other types.
/// By default, this allows conversions essentially
/// being: [Option]\<T> -> [Option]\<U> or
/// [Result]<T, E> -> [Result]<U, R>
///
/// The main benefit here is that you don't need to
/// take the value out of the type in order to convert it.
/// Meaning that you don't need to handle the different paths
/// of match statements, while also making your code
/// easier to read.
/// # Implementation
/// If you make a type which contains others,
/// it may be worth it to implement this for it
/// if you expect it to be converted a lot.
///
/// For example, let's say that we have a version
/// of an [Option] which we are making for a library
/// and want people to be able to convert it easily.
/// You could:
///```
/// # use abes_nice_things::AsFrom;
/// # #[derive(Debug, PartialEq)]
/// enum WaitOption<T> {
///     Some(T),
///     None,
///     NotYet
/// }
/// impl<T: From<U>, U> AsFrom<WaitOption<U>> for WaitOption<T> {
///     fn as_from(value: WaitOption<U>) -> Self {
///         match value {
///             WaitOption::Some(val) => return WaitOption::Some(T::from(val)),
///             WaitOption::None => return WaitOption::None,
///             WaitOption::NotYet => return WaitOption::NotYet,
///         }
///     }
/// }
/// # fn main() {
/// #   let some: WaitOption<&str> = WaitOption::Some("I fuckin hate lemons");
/// #   let some_conv = <WaitOption<String>>::as_from(some);
/// #   let none: WaitOption<&str> = WaitOption::None;
/// #   let none_conv = <WaitOption<String>>::as_from(none);
/// #   let yet: WaitOption<&str> = WaitOption::NotYet;
/// #   let yet_conv = <WaitOption<String>>::as_from(yet);
/// #   assert_eq!(some_conv, WaitOption::Some(String::from("I fuckin hate lemons")));
/// #   assert_eq!(none_conv, WaitOption::None);
/// #   assert_eq!(yet_conv, WaitOption::NotYet);
/// # }
///```
pub trait AsFrom<T> {
    /// A function for converting inside another type.
    /// For example, instead of having to set
    /// up this in order to convert inside an [Option]:
    ///```
    /// fn convert(src: Option<[u8; 5]>) -> Option<Vec<u8>> {
    ///     match src {
    ///         Some(source) => return Some(Vec::<u8>::from(source)),
    ///         None => return None
    ///     }
    /// }
    /// # fn main() {
    /// #   let src = [1, 2, 3, 4, 6];
    /// #   assert_eq!(convert(Some(src)), Some(Vec::from(src)));
    /// # }
    ///```
    /// You could instead simply
    ///```
    /// # use abes_nice_things::AsFrom;
    /// fn convert(src: Option<[u8; 5]>) -> Option<Vec<u8>> {
    ///     return <Option<Vec<u8>>>::as_from(src);
    /// }
    /// # fn main() {
    /// #   let src = [1, 1, 3, 4, 5];
    /// #   assert_eq!(convert(Some(src)), Some(Vec::from(src)));
    /// # }
    ///```
    /// While in this case, with a infallible conversion inside an [Option],
    /// this is not removing that much complexity, it is still useful.
    ///
    /// However, with a [Result], the complexity avoided is a bit worse.
    /// Instead of:
    ///```
    /// fn convert(src: Result<[u8; 5], &str>) -> Result<Vec<u8>, String> {
    ///     match src {
    ///         Ok(val) => return Ok(<Vec<u8>>::from(val)),
    ///         Err(error) => return Err(String::from(error))
    ///     }
    /// }
    /// # fn main() {
    /// #   let ok_src = [5, 27, 53, 92, 127];
    /// #   let err_src = "we're fucked";
    /// #   assert_eq!(convert(Ok(ok_src)), Ok(ok_src.into()));
    /// #   assert_eq!(convert(Err(err_src)), Err(err_src.into()));
    /// # }
    ///```
    /// You can:
    ///```
    /// # use abes_nice_things::AsFrom;
    /// fn convert(src: Result<[u8; 5], &str>) -> Result<Vec<u8>, String> {
    ///     return <Result<Vec<u8>, String>>::as_from(src)
    /// }
    /// # fn main() {
    /// #   let ok_src = [230, 76, 120, 184, 37];
    /// #   let err_src = "abc";
    /// #   assert_eq!(convert(Ok(ok_src)), Ok(ok_src.into()));
    /// #   assert_eq!(convert(Err(err_src)), Err(err_src.into()));
    /// # }
    ///```
    fn as_from(value: T) -> Self;
}
impl<T: From<U>, U> AsFrom<Option<U>> for Option<T> {
    fn as_from(value: Option<U>) -> Self {
        match value {
            Some(value) => return Some(T::from(value)),
            None => return None,
        }
    }
}
impl<T: From<U>, U, E: From<R>, R> AsFrom<Result<U, R>> for Result<T, E> {
    fn as_from(value: Result<U, R>) -> Self {
        match value {
            Ok(value) => return Ok(T::from(value)),
            Err(error) => return Err(E::from(error)),
        }
    }
}
/// A trait for attempting to convert within another type.
/// This shares a similar relationship to [AsFrom]
/// as [TryFrom] does to [From].
/// In other words, it is for when it is not guaranteed
/// that the conversion will succeed.
/// If it is a guarantee that it will succeed,
/// you should just use [AsFrom].
/// 
/// For more information, see [AsFrom] and [TryFrom]
pub trait AsTryFrom<T>
where
    Self: Sized,
{
    type Error;
    /// This is a version of [AsFrom] which allows
    /// for the conversion to return errors, much
    /// like [TryFrom]. To be perfectly blunt, reading
    /// their documentation should give you a good idea
    /// on how to use this.
    fn as_try_from(value: T) -> Result<Self, Self::Error>;
}
impl<T: TryFrom<U>, U> AsTryFrom<Option<U>> for Option<T> {
    type Error = T::Error;
    fn as_try_from(value: Option<U>) -> Result<Self, Self::Error> {
        match value {
            Some(value) => match T::try_from(value) {
                Ok(value) => return Ok(Some(value)),
                Err(error) => return Err(error),
            },
            // TODO: determine if this is correct or if it should panic
            None => return Ok(None),
        }
    }
}
impl<T: TryFrom<U>, U, E: TryFrom<R>, R> AsTryFrom<Result<U, R>> for Result<T, E> {
    type Error = Either<T::Error, E::Error>;
    fn as_try_from(value: Result<U, R>) -> Result<Self, Self::Error> {
        match value {
            Ok(value) => match T::try_from(value) {
                Ok(value) => return Ok(Ok(value)),
                Err(fail) => return Err(Self::Error::new_t(fail)),
            },
            Err(error) => match E::try_from(error) {
                Ok(error) => return Ok(Err(error)),
                Err(fail) => return Err(Self::Error::new_u(fail)),
            },
        }
    }
}
/// This is the sibling trait for
/// [AsFrom], it is implemented
/// automatically and should **NOT**
/// be manually implemented.
/// Instead, implement [AsFrom]
pub trait AsInto<T> {
    /// This is very similar to [Into].
    /// However, much like [AsFrom], this
    /// does its operation on the value inside
    /// other types such as [Option] or [Result].
    /// The best way to explain it is that it can
    /// convert [Option<T>] to [Option<U>] and
    /// [Result<T, E>] to [Result<U, R>] in a
    /// concise way while still handling all
    /// variant possiblilities properly.
    /// Without this, in order to convert
    /// [Option<T>] to [Option<U>], you would need
    /// ```
    /// fn into<T: Into<U>, U>(from: Option<T>) -> Option<U> {
    ///     match from {
    ///         Some(inner) => return Some(inner.into()),
    ///         None => None
    ///     }
    /// }
    /// ```
    /// While that admittedly is not that complex,
    /// it would be worse for [Result] and in both
    /// cases, it would be better if you didn't have
    /// to think about it.
    ///
    /// That is where [AsInto] comes in, it shortens
    /// it from that to
    /// ```ignore
    /// fn into<T: Into<U>, U>(from: Option<T>) -> Option<U> {
    ///     from.as_into()
    /// }
    /// ```
    /// For more information, see [Into] and [AsFrom]
    fn as_into(self) -> T;
}
impl<T: AsFrom<U>, U> AsInto<T> for U {
    fn as_into(self) -> T {
        T::as_from(self)
    }
}
/// This is the sibling trait for
/// [AsTryFrom], it is implemented
/// automatically and should **NOT**
/// be manually implemented.
/// Instead, implement [AsTryFrom]
pub trait AsTryInto<T> {
    type Error;
    /// This is similar to [AsTryFrom] and
    /// [TryInto]. This is able to attempt
    /// to convert the value inside another
    /// type while allowing for error
    /// returning if it fails.
    /// Notably, if it does fail, it will
    /// not return the error directly, because
    /// there are two different possible error
    /// types. Instead it will return [Either]
    /// T's error type or U's error type.
    /// The easiest way to extract the error from
    /// that [Either] is a match statement.
    ///
    /// For more information, see [as_try_from](AsTryFrom::as_try_from)
    /// and [try_into](TryInto::try_into)
    fn as_try_into(self) -> Result<T, Self::Error>;
}
impl<T: AsTryFrom<U>, U> AsTryInto<T> for U {
    type Error = T::Error;
    fn as_try_into(self) -> Result<T, Self::Error> {
        T::as_try_from(self)
    }
}
/// This is a type that will either be T or U.
/// That's it, it's pretty simple.
/// Because there is no way to guarantee that
/// the two types are not the same, it cannot
/// implement [From] or [Into].
pub enum Either<T, U> {
    T(T),
    U(U),
}
impl<T, U> Either<T, U> {
    /// Creates a new instace of [Either]
    /// with the T variant
    pub fn new_t(t: T) -> Self {
        Either::T(t)
    }
    /// Creates a new instance of [Either]
    /// with the U variant
    pub fn new_u(u: U) -> Self {
        Either::U(u)
    }
}
impl<T: From<Y>, U: From<I>, Y, I> AsFrom<Either<Y, I>> for Either<T, U> {
    fn as_from(value: Either<Y, I>) -> Self {
        match value {
            Either::T(val) => Either::T(T::from(val)),
            Either::U(val) => Either::U(U::from(val))
        }
    }
}
