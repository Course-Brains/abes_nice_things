use abes_nice_things::FromBinary;
use abes_nice_things::aes::DecryptReader;
use std::io::Read;

fn main() {
    let mut initialization_vector = [0; 16];
    let mut key = [0; 240];
    let mut key_file = std::fs::File::open("key").unwrap();
    key_file.read_exact(&mut initialization_vector).unwrap();
    key_file.read_exact(&mut key).unwrap();

    let mut reader = DecryptReader::new(
        std::fs::File::open("cipher").unwrap(),
        initialization_vector,
        &key,
    )
    .expect("Invalid key");
    let mut decrypted = vec![0; usize::from_binary(&mut reader).unwrap()];
    reader.read_exact(&mut decrypted).unwrap();

    std::fs::write("decrypted", decrypted).unwrap();
}
