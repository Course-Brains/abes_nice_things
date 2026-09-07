use abes_nice_things::ToBinary;
use abes_nice_things::aes::EncryptWriter;
use std::io::Read;
use std::io::Write;

fn main() {
    // total key: 256 bytes
    let mut initialization_vector = [0; 16];
    let mut key = [0; 240];
    let mut source = std::fs::File::open("/dev/random").unwrap();
    source.read_exact(&mut initialization_vector).unwrap();
    source.read_exact(&mut key).unwrap();
    let mut key_file = std::fs::File::create("key").unwrap();
    key_file.write_all(&initialization_vector).unwrap();
    key_file.write_all(&key).unwrap();

    let message = std::fs::read("message").unwrap();
    let mut output = EncryptWriter::new(
        std::fs::File::create("cipher").unwrap(),
        initialization_vector,
        &key,
    )
    .unwrap();
    message.len().to_binary(&mut output).unwrap();
    output.write_all(message.as_slice()).unwrap();
}
