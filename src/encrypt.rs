use abes_nice_things::ToBinary;
use abes_nice_things::aes::EncryptWriter;
use std::io::Read;
use std::io::Write;

fn main() {
    let mut key = [0; 240];
    std::fs::File::open("/dev/random")
        .unwrap()
        .read_exact(&mut key)
        .unwrap();
    std::fs::write("key", &key).unwrap();

    let message = std::fs::read("message").unwrap();
    let mut output = EncryptWriter::new(std::fs::File::create("cipher").unwrap(), &key);
    message.len().to_binary(&mut output).unwrap();
    output.write_all(message.as_slice()).unwrap();
}
