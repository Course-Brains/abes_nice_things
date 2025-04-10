use albatrice::{manual_reader, input};
fn main() {
    println!("What file?");
    let path = input();
    let file = std::fs::File::open(path).unwrap();
    manual_reader(file).unwrap();
}
