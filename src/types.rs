use std::sync::Mutex;

pub struct OnceLockMethod<'a, T> {
    inner: Mutex<Option<T>>,
    method: &'a (dyn Fn() -> T + Sync),
}
impl<'a, T> OnceLockMethod<'a, T> {
    pub const fn new(method: &'a (impl Fn() -> T + Sync)) -> OnceLockMethod<'a, T> {
        return OnceLockMethod {
            inner: Mutex::new(None),
            method,
        }
    }
    pub fn init(&self) {
        *self.inner.lock().unwrap() = Some((self.method)());
    }
    pub fn get(&self) -> Option<T>
    where T: Clone
    {
        return (*self.inner.lock().unwrap()).clone()
    }
    pub fn get_unsafe(&self) -> T
    where T: Clone
    {
        return self.get().unwrap()
    }
    pub fn unwrap_none(&self) -> bool
    where T: Clone
    {
        if let Some(_) = self.get() {
            panic!("Value was defined");
        }
        return true
    }
    pub fn get_or_init(&self) -> T
    where T: Clone
    {
        if let None = self.get() {
            self.init();
        }
        return self.get_unsafe()
    }
}
