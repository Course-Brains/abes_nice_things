use abes_nice_things::{input, manual_reader};
fn main() {
    println!("What file?");
    let path = input();
    let file = std::fs::File::open(path).unwrap();
    manual_reader(file).unwrap();
}
