use abes_nice_things::FromBinary;
use abes_nice_things::aes::DecryptReader;
use std::io::Read;

fn main() {
    let key = std::fs::read("key").unwrap();
    let mut reader = DecryptReader::new(std::fs::File::open("cipher").unwrap(), &key);
    let mut decrypted = vec![0; usize::from_binary(&mut reader).unwrap()];
    reader.read_exact(&mut decrypted).unwrap();

    std::fs::write("decrypted", decrypted).unwrap();
}
