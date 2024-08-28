use crate::Either;
pub trait AsFrom<T> {
    fn as_from(value: T) -> Self;
}
impl<T: From<U>, U> AsFrom<Option<U>> for Option<T> {
    fn as_from(value: Option<U>) -> Self {
        match value {
            Some(value) => return Some(T::from(value)),
            None => return None
        }
    }
}
impl<T: From<U>, U, E: From<R>, R> AsFrom<Result<U, R>> for Result<T, E> {
    fn as_from(value: Result<U, R>) -> Self {
        match value {
            Ok(value) => return Ok(T::from(value)),
            Err(error) => return Err(E::from(error))
        }
    }
}
pub trait AsTryFrom<T>
where Self: Sized
{
    type Error;
    fn as_try_from(value: T) -> Result<Self, Self::Error>;
}
impl<T: TryFrom<U>, U> AsTryFrom<Option<U>> for Option<T> {
    type Error = T::Error;
    fn as_try_from(value: Option<U>) -> Result<Self, Self::Error> {
        match value {
            Some(value) => {
                match T::try_from(value) {
                    Ok(value) => return Ok(Some(value)),
                    Err(error) => return Err(error)
                }
            }
            // TODO: determine if this is correct or if it should panic
            None => return Ok(None)
        }
    }
    
}
impl<T: TryFrom<U>, U, E: TryFrom<R>, R> AsTryFrom<Result<U, R>> for Result<T, E> {
    type Error = Either<T::Error, E::Error>;
    fn as_try_from(value: Result<U, R>) -> Result<Self, Self::Error> {
        match value {
            Ok(value) => {
                match T::try_from(value) {
                    Ok(value) => return Ok(Ok(value)),
                    Err(fail) => return Err(Self::Error::new_t(fail))
                }
            }
            Err(error) => {
                match E::try_from(error) {
                    Ok(error) => return Ok(Err(error)),
                    Err(fail) => return Err(Self::Error::new_u(fail))
                }
            }
        }
    }
}
pub trait AsInto<T> {
    fn as_into(self) -> T;
}
impl<T: Into<U>, U> AsInto<Option<U>> for Option<T> {
    fn as_into(self) -> Option<U> {
        match self {
            Some(value) => return Some(value.into()),
            None => return None
        }
    }
}
impl<T: Into<U>, U, E: Into<R>, R> AsInto<Result<U, R>> for Result<T, E> {
    fn as_into(self) -> Result<U, R> {
        match self {
            Ok(value) => return Ok(value.into()),
            Err(error) => return Err(error.into())
        }
    }
}
pub trait AsTryInto<T> {
    type Error;
    fn as_try_into(self) -> Result<T, Self::Error>;
}
impl<T: TryInto<U>, U> AsTryInto<Option<U>> for Option<T> {
    type Error = T::Error;
    fn as_try_into(self) -> Result<Option<U>, Self::Error> {
        match self {
            Some(value) => {
                match value.try_into() {
                    Ok(value) => return Ok(Some(value)),
                    Err(fail) => return Err(fail)
                }
            }
            // TODO: find out if this is the correct behavior
            None => return Ok(None)
        }
    }
}
impl<T: TryInto<U>, U, E: TryInto<R>, R> AsTryInto<Result<U, R>> for Result<T, E> {
    type Error = Either<T::Error, E::Error>;
    fn as_try_into(self) -> Result<Result<U, R>, Self::Error> {
        match self {
            Ok(value) => {
                match value.try_into() {
                    Ok(value) => return Ok(Ok(value)),
                    Err(fail) => return Err(Self::Error::new_t(fail))
                }
            }
            Err(error) => {
                match error.try_into() {
                    Ok(error) => return Ok(Err(error)),
                    Err(fail) => return Err(Self::Error::new_u(fail))
                }
            }
        }
    }
}