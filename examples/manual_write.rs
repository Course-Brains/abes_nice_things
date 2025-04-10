use albatrice::{manual_writer, input};
fn main() {
    println!("What file?");
    let path = input();
    let file = std::fs::File::create(path).unwrap();
    manual_writer(file).unwrap();
}
