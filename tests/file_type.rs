#[cfg(test)]
mod tests {
    #[allow(non_snake_case)]
    mod VALID_FILE_TYPE_VALUES {
        use abes_nice_things::file_ops::{FileType, VALID_FILE_TYPE_VALUES as VALUES};
        use std::path::PathBuf;
        #[test]
        fn equivalence() {
            println!("Values:({:?})", VALUES);
            for (key, value) in VALUES.iter() {
                let path: PathBuf = PathBuf::new().join("test.".to_owned() + key);
                let file_type: FileType = FileType::from_path(&path);
                assert_eq!(
                    &file_type, value,
                    "FileType generation from path was incorrect"
                )
            }
        }
        mod equivalence {
            use super::{FileType, PathBuf, VALUES};
            #[test]
            fn toml() {
                sub("toml", FileType::Toml);
            }
            #[test]
            fn bin() {
                sub("bin", FileType::Bin);
            }

            fn sub(str: &str, ft: FileType) {
                assert!(
                    VALUES.contains(&(str, ft)),
                    "Values list does not contain type"
                );
                assert_eq!(
                    FileType::from_path(&PathBuf::new().join("test.".to_owned() + str)),
                    ft,
                    "from_path created incorrect value"
                )
            }
        }
    }
}
