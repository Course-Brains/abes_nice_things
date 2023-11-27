#[cfg(test)]
mod tests {
    use ant::types::OnceLockMethod;
    #[test]
    fn get_returns_none_on_undefined() {
        let method = || -> String { return "Hello".to_string() };
        let lock: OnceLockMethod<String> = OnceLockMethod::new(&method);
        assert_eq!(lock.get(), None, "Lock had value on creation")
    }
    #[test]
    fn get_returns_value_on_defined() {
        let value: bool = true;
        let method = || -> bool { return true };
        let lock: OnceLockMethod<bool> = OnceLockMethod::new(&method);
        lock.init();
        assert_eq!(lock.get(), Some(value), "Initial value and definition were inqeivalent")
    }
    #[test]
    #[should_panic]
    fn get_unsafe_fails_on_undefined() {
        let method = || -> isize { return -27 };
        let lock: OnceLockMethod<isize> = OnceLockMethod::new(&method);
        lock.get_unsafe();
    }
    #[test]
    fn get_unsafe_pass_on_defined() {
        let value: usize = 5;
        let method = || -> usize { return 5 };
        let lock: OnceLockMethod<usize> = OnceLockMethod::new(&method);
        assert!(lock.unwrap_none(), "Lock had value on creation");
        lock.init();
        assert_eq!(lock.get_unsafe(), value, "Initial value and definition were inequivalent");
    }
    #[test]
    #[should_panic]
    fn unwrap_none_fails_on_defined() {
        let value: u8 = 12;
        let method = || -> u8 { return 12 };
        let lock: OnceLockMethod<u8> = OnceLockMethod::new(&method);
        assert!(lock.unwrap_none(), "Lock had value on creation");
        lock.init();
        assert_eq!(lock.get_unsafe(), value, "Initial value and definition were ineqivalent");
        lock.unwrap_none();
    }
    #[test]
    fn get_or_init_creates_value_on_undefined() {
        let value: f32 = -27.6;
        let method = || -> f32 { return -27.6 };
        let lock: OnceLockMethod<f32> = OnceLockMethod::new(&method);
        assert!(lock.unwrap_none(), "Lock had value on creation");
        assert_eq!(lock.get_or_init(), value, "Initial value and definition were inequivalent");
    }
    #[test]
    fn get_or_init_returns_value_on_defined() {
        let value: i16 = -35;
        let method = || -> i16 { return -35 };
        let lock: OnceLockMethod<i16> = OnceLockMethod::new(&method);
        assert!(lock.unwrap_none(), "Lock had value on creation");
        lock.init();
        assert_eq!(lock.get_or_init(), value, "Initial value and definition were ineqivalent");
    }
}