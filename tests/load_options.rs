#[cfg(test)]
mod tests {
    #[allow(non_snake_case)]
    mod FileOptions {
        use ant::file_ops::*;
        use sequential_test::sequential;
        use serde::{Serialize, Deserialize};
        
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct Simple {
            x: usize
        }

        #[test]
        #[sequential]
        fn simple_save_load_test() {
            println!("Creating value");
            let value: Simple = Simple { x: 24 };
            println!("Creating FileOptions instance");
            let options: FileOptions = FileOptions::new();
            println!("Saving value");
            save_toml("test.toml", &value);
            println!("Loading value");
            let load: Simple = load_toml("test.toml");
            println!("Comparing equivalence");
            assert_eq!(value, load, "Initial and loaded value were inequivalent: initial:{:?}, load:{:?}", value, load);
            println!("Deleting file");
            delete("test.toml");
            println!("Where problem?");
        }
    }
}