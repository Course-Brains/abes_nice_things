fn main() {
    let mut args = std::env::args();
    args.next();
    if std::fs::exists("run").unwrap() {
        //println!("Using run script");
        std::process::Command::new("./run").args(args).spawn()
    } else if std::fs::exists("Cargo.toml").unwrap() {
        //println!("Using cargo");
        std::process::Command::new("cargo")
            .arg("run")
            .arg("--release")
            .args(args)
            .spawn()
    } else {
        panic!("Could not find run script or Cargo.toml");
    }
    .unwrap()
    .wait()
    .unwrap();
}
