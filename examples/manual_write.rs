use abes_nice_things::{input, manual_writer};
fn main() {
    println!("What file?");
    let path = input();
    let file = std::fs::File::create(path).unwrap();
    manual_writer(file).unwrap();
}
