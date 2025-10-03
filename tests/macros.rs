#[cfg(test)]
mod tests {
    mod assert_pattern {
        use abes_nice_things::assert_pattern;
        #[test]
        fn success() {
            let value: Option<usize> = None;
            assert_pattern!(value, None);
        }
        #[test]
        fn success_message() {
            let value: Option<usize> = None;
            assert_pattern!(value, None, "this");
        }
        #[test]
        #[should_panic(expected = "Item did not match variant")]
        fn fail() {
            let value: Option<usize> = Some(64);
            assert_pattern!(value, None);
        }
        #[test]
        #[should_panic(expected = "is")]
        fn fail_message() {
            let value: Option<usize> = Some(64);
            assert_pattern!(value, None, "is");
        }
    }
    mod assert_pattern_ne {
        use abes_nice_things::assert_pattern_ne;
        #[test]
        fn success() {
            let value: Option<usize> = Some(64);
            assert_pattern_ne!(value, None);
        }
        #[test]
        fn success_message() {
            let value: Option<usize> = Some(64);
            assert_pattern_ne!(value, None, "Completely");
        }
        #[test]
        #[should_panic(expected = "Item matched variant")]
        fn fail() {
            let value: Option<usize> = None;
            assert_pattern_ne!(value, None);
        }
        #[test]
        #[should_panic(expected = "Unnecessary")]
        fn fail_message() {
            let value: Option<usize> = None;
            assert_pattern_ne!(value, None, "Unnecessary");
        }
    }
}
