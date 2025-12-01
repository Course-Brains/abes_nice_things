if [ "$EXTRACT" != "" ]; then
    mkdir working_dir || exit 1
    cd working_dir
else
    mkdir $SCRIPT_NAME || exit 1
    cd $SCRIPT_NAME
fi
echo "
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
const PATH: &str = \"../$SCRIPT_NAME.bash\";
fn main() {
    let mut script = std::fs::File::open(PATH).unwrap();
    println!(\"Source file length: {}\", std::fs::metadata(PATH).unwrap().len());
    script.seek(SeekFrom::End(-8)).unwrap();
    let mut buf64 = [0_u8; 8];
    script.read_exact(&mut buf64).unwrap();
    let start_point = u64::from_le_bytes(buf64);
    println!(\"Start point: {start_point}\");
    script.seek(SeekFrom::Start(start_point)).unwrap();

    let mut buf32 = [0_u8; 4];
    let mut buf8 = [0_u8];
    let mut buf;
    while script.read_exact(&mut buf64).is_ok() {
	let file_len = u64::from_le_bytes(buf64);
	script.read_exact(&mut buf32).unwrap();
	let path_len = u32::from_le_bytes(buf32);
	script.read_exact(&mut buf8).unwrap();
	let executable = match u8::from_le_bytes(buf8) {
	    0 => false,
	    1 => true,
	    _ => unreachable!(\"INVALID FORMAT\"),
	};
	buf = vec![0_u8; path_len as usize];
	script.read_exact(&mut buf).unwrap();
	let path = String::from_utf8(buf).unwrap();
	println!(\"unpacking {path}\");
	std::fs::create_dir_all(PathBuf::from(path.clone()).parent().unwrap()).unwrap();
	buf = vec![0_u8; file_len as usize];
	script.read_exact(&mut buf).unwrap();
	std::fs::write(path.clone(), &buf).unwrap();
	if executable {
	    std::process::Command::new(\"chmod\").arg(\"+x\").arg(path.as_str()).output().unwrap();
	}
    }
}
" > extractor.rs
rustc extractor.rs --edition 2024 -o extractor  > /dev/null 2> /dev/null || exit 1
./extractor 2> /dev/null
rm extractor.rs
rm extractor

if [ "$EXTRACT" != "" ]; then
    cargo build --release
    mv target/release/$EXTRACT .. || exit 1
    cd ..
    rm -r working_dir
fi
