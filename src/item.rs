use crate::{FromBinary, ToBinary, Binary};
use std::path::Path;
/// This is a read only type for pulling things
/// from storage but not taking up a lot of
/// memory when they aren't needed.
pub struct Item<T: FromBinary, P: AsRef<Path>> {
    pub source: P,
    value: Option<Box<T>>
}
impl<T: FromBinary, P: AsRef<Path>> Item<T, P> {
    pub const fn new(source: P) -> Self {
        Item {
            source,
            value: None
        }
    }
    /// Will do nothing if it has the loaded
    /// value already
    pub fn load(&mut self) {
        if self.value.is_none() {
            self.load_unchecked()
        }
    }
    /// Does not cause undefined behavior,
    /// just is inefficient if it has already
    /// been initialized because it will still
    /// pull the data from the file anyway.
    pub fn load_unchecked(&mut self) {
        let mut int = Some(
            Box::new(
                T::from_binary(
                    &mut std::fs::File::open(self.source.as_ref().to_path_buf()).unwrap()
                )
            )
        );
        std::mem::swap(&mut int, &mut self.value);
    }
    /// Gets a reference to the value
    /// and loads it from storage if
    /// necessary.
    pub fn get(&mut self) -> &T {
        self.load();
        self.value.as_ref().unwrap()
    }
    /// Will cause undefined behavior if the
    /// data has not been loaded yet.
    pub const unsafe fn get_unchecked(&self) -> &T {
        self.value.as_ref().unwrap_unchecked()
    }
    /// Removes the data from memory and requires
    /// it to be loaded again before use
    pub fn drop(&mut self) {
        self.value = None;
    }
}
impl<T: Binary, P: AsRef<Path>> Item<T, P> {
    pub fn save(&self, value: T) {
        value.to_binary(
            &mut std::fs::File::create(
                self.source.as_ref().to_path_buf()
            ).unwrap()
        )
    }
}
impl<T: FromBinary, P: AsRef<Path> + FromBinary> FromBinary for Item<T, P> {
    fn from_binary(binary: &mut dyn std::io::Read) -> Self {
        Item {
            source: P::from_binary(binary),
            value: Option::from_binary(binary)
        }
    }
}
impl<T: Binary, P: AsRef<Path> + ToBinary> ToBinary for Item<T, P> {
    fn to_binary(&self, binary: &mut dyn std::io::Write) {
        self.source.to_binary(binary);
        self.value.to_binary(binary);
    }
}