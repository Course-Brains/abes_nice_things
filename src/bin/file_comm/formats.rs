use std::net::TcpStream;
use abes_nice_things::{ToBinary, FromBinary};
use crate::{Settings, Mode, quiet};

pub type FormatID = u32;
pub const HIGHEST: FormatID = 1;

pub fn hand_shake(stream: TcpStream, settings: Settings) -> Result<(), String> {
    println!("Beginning format handshake");
    //////////////////////////////////////////////////////////////
    // The person sending will be the one to suggest the format //
    //////////////////////////////////////////////////////////////
    // We are assuming that backwards compatability exists      //
    //////////////////////////////////////////////////////////////
    match settings.mode {
        Mode::Recv => recv_hand_shake(stream, settings),
        Mode::Send => send_hand_shake(stream, settings)
    }
}
fn send_hand_shake(mut stream: TcpStream, settings: Settings) -> Result<(), String> {
    quiet!("Sending suggested format: {}", settings.get_format());
    settings.get_format().to_binary(&mut stream).unwrap();
    // Format is decided by the reciever
    let format = Result::<FormatID, String>::from_binary(&mut stream).unwrap()?;
    quiet!("Decided to use format: {format}\nFormat handshake done");
    match format {
        0 => f0::send(stream, settings),
        1 => f1::send(stream, settings),
        _ => return Err("Invalid format given by other".to_string())
    }
    Ok(())
}
fn recv_hand_shake(mut stream: TcpStream, settings: Settings) -> Result<(), String> {
    quiet!("Waiting for suggested format");
    let other_highest = FormatID::from_binary(&mut stream).unwrap();
    quiet!("Suggestion: {other_highest}");
    let format = {
        // We are able to process their highest format
        if other_highest <= settings.get_format() {
            quiet!("Accepting suggestion");
            other_highest
        }
        // We cannot match them, so they have to match us
        else {
            quiet!("Suggestion is impossible, sending alternative");
            settings.get_format()
        }
    };
    Ok::<u32, String>(format).to_binary(&mut stream).unwrap();
    quiet!("Format handshake done");
    match format {
        0 => f0::recv(stream, settings),
        1 => f1::recv(stream, settings),
        _ => unreachable!()
    };
    Ok(())
}
mod f0 {
    // Format:
    // Length of name(u32)
    // name
    // data
    use std::{
        net::TcpStream,
        fs::File,
        path::PathBuf,
        io::{Read, Write}
    };
    use crate::{Settings, quiet};
    use abes_nice_things::{input, Input, ToBinary, FromBinary};

    static mut TO_SEND: Option<PathBuf> = None;
    pub fn send(mut stream: TcpStream, settings: Settings) {
        let file;
        #[allow(static_mut_refs)]// compiler is wrong
        let path = match unsafe { TO_SEND.clone() } {
            Some(path) => {
                file = File::open(&path).unwrap();
                path
            },
            None => {
                loop {
                    let path = match settings.path {
                        Some(ref path) => path,
                        None => {
                            println!("What file do you want to send?");
                            &input()
                        }
                    };
                    match File::open(path) { 
                        Ok(file_in) => {
                            quiet!("Valid file");
                            file = file_in;
                            if settings.host.is_some() {
                                if settings.path.is_some() {}
                                else if *"y" == <Input>::yn()
                                    .msg("Do you want to use this for subsequent requests?y/n")
                                .get().unwrap() {
                                    unsafe { TO_SEND = Some(PathBuf::from(&path)) }
                                }
                            }
                            break PathBuf::from(path)
                        }
                        Err(error) => eprintln!("Failed to identify file validity: {error}")
                    }
                }
            }
        };
        let path = path.file_name().expect("Failed to get file name").to_str().unwrap();
        let len = file.metadata().expect("Failed to get file metadata").len();

        quiet!("Sending metadata");
        (path.len() as u32).to_binary(&mut stream).unwrap();
        stream.write_all(path.as_bytes()).unwrap();
        quiet!("Sending file");
        transfer(file, stream, len, 1000);
        quiet!("File sent")
    }
    pub fn recv(mut stream: TcpStream, settings: Settings) {
        quiet!("Getting metadata");
        let name_len = u32::from_binary(&mut stream).unwrap();
        let mut buf = vec![0; name_len as usize];
        stream.read_exact(&mut buf).unwrap();
        let name = String::from_utf8(buf).unwrap();
        if settings.auto_accept {}
        else if "n" == match stream.peer_addr() {
            Ok(addr) => <Input>::yn().msg(
                &format!(
                    "Are you sure you want to accept {name} from {addr}?y/n"
                )
            ).get().unwrap(),
            Err(_) => <Input>::yn().msg(
                &format!(
                    "Are you sure you want to accept {name} from unknown?y/n"
                )
            ).get().unwrap()
        } {
            return
        }
        let mut buf = Vec::new();
        quiet!("Getting data");
        stream.read_to_end(&mut buf).unwrap();
        quiet!("Writing to file");
        std::fs::write(name, buf).unwrap();
        quiet!("Done")
        
    }
    pub fn transfer(mut from: impl Read, mut to: impl Write, mut len: u64, interval: usize) {
        while len > interval as u64 {
            let mut buf = vec![0_u8; interval];
            from.read_exact(&mut buf).unwrap();
            to.write_all(&buf).unwrap();
            len -= interval as u64;
        }
        let mut buf = vec![0; len.try_into().unwrap()];
        from.read_exact(&mut buf).unwrap();
        to.write_all(&buf).unwrap();
    }
}
mod f1 {
    // Format:
    // 1: Len: u64
    // 2: len: u32 of name
    // 3: name: String
    // 4: data: [u8]
    use std::net::TcpStream;
    use std::io::{Read, Write};
    use crate::{quiet, Settings, QUIET};
    use abes_nice_things::{FromBinary, ToBinary, Input, input};
    // Assumed to be less than 2^32
    const BUFFER_SIZE: u64 = 1000;
    // A thing for keeping track of how much has been written
    // since the last print, and when that was
    struct Tracker {
        written: u64,
        total_written: u64,
        previous: Option<std::time::Instant>
    }
    impl Tracker {
        // The threshold for when we should print again
        const THRESHOLD: std::time::Duration = std::time::Duration::from_millis(500);
        const BAR_LENGTH: usize = 50;
        fn display(&mut self, total: u64) {
            if unsafe { QUIET } {
                return
            }
            if let Some(previous) = self.previous {
                // Get the time elapsed and check if we are printing
                let diff = previous.elapsed();
                if !(diff > Tracker::THRESHOLD) {
                    // We should not print yet
                    return;
                }
                // Important that we don't let go of stdout for this
                let mut stdout = std::io::stdout().lock();
                // Delete the current line
                stdout.write_fmt(format_args!("\r")).unwrap();
                // Assuming that it will be close enough to 1 second
                let rate = Rate(self.written);
                let percent_done = match self.total_written {
                    0 => 0.0,
                    written => (written as f64)/(total as f64)
                };
                let filled = ((Tracker::BAR_LENGTH as f64) * percent_done) as usize;
                let unfilled = Tracker::BAR_LENGTH - filled;
                stdout.write(
                    ("[".to_string() +
                    &"#".repeat(filled) +
                    &"-".repeat(unfilled) +
                    &"] ").as_bytes()
                ).unwrap();
                stdout.write_fmt(format_args!("%{:.3} ", percent_done*100.0)).unwrap();
                stdout.write_fmt(format_args!("{rate}")).unwrap();
                stdout.flush().unwrap();
            }
            self.written = 0;
            self.previous = Some(std::time::Instant::now());
        }
    }
    struct Rate(u64);
    impl std::fmt::Display for Rate {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.0 < 1000 {
                // Showing in bytes
                write!(f, "{} B/s", self.0)
            }
            else if self.0 < 1000000 {
                // Showing in kilo bytes
                write!(f, "{} kB/s", self.0/1000) 
            }
            else {
                // Showing in mega bytes
                write!(f, "{} mB/s", self.0/1000000)
            }
            // I'm not accounting for giga bytes per second, fuck that
        }
    }
    fn transfer(mut from: impl Read, mut to: impl Write, len: u64, tracker: &mut Tracker) {
        let mut buf = const { [0_u8; BUFFER_SIZE as usize] };
        let mut remaining = len;
        while remaining > BUFFER_SIZE {
            //std::thread::sleep(std::time::Duration::from_millis(10));
            //println!("Transfering section of size: {BUFFER_SIZE}: {remaining} remaining");
            // First the buffer sized chunks
            from.read_exact(&mut buf).unwrap();
            to.write_all(&mut buf).unwrap();
            remaining -= BUFFER_SIZE;
            tracker.written += BUFFER_SIZE;
            tracker.total_written += BUFFER_SIZE;
            tracker.display(len);
        }
        // Then the remainder
        //println!("Transfering last section of len: {remaining}");
        let mut buf = vec![0_u8; remaining as usize];
        from.read_exact(&mut buf).unwrap();
        to.write_all(&mut buf).unwrap();
        //println!("Done transfering");
    }
    pub fn send(mut stream: TcpStream, settings: Settings) {
        let path = settings.path.unwrap_or_else(|| {
            // Getting a path that exists
            loop {
                println!("What file do you want to send?");
                let path = input();
                match std::fs::exists(&path) {
                    Ok(true) => break path,
                    Ok(false) => println!("That file does not exist"),
                    Err(error) => eprintln!(
                        "Failed to determine if the file exists: {error}"
                    )
                }
            }
        });
        if !settings.auto_accept && "n" == <Input>::yn()
            .msg(&format!("Do you want to send {path}?y/n"))
        .get().unwrap() {
            return
        }
        quiet!("Getting file at path: {path}");
        let file = std::fs::File::open(&path).unwrap();
        // Sending over the length of the file to be sent (1)
        let len = file.metadata().unwrap().len();
        // Sending the path
        let binding = std::path::PathBuf::from(path);
        let path = binding.file_name().unwrap().to_str().unwrap();
        path.to_binary(&mut stream).unwrap();
        quiet!("File is length: {len}");
        len.to_binary(&mut stream).unwrap();

        // Sending over the data
        quiet!("Sending data\n");
        let mut tracker = Tracker {
            written: 0,
            total_written: 0,
            previous: None
        };
        transfer(file, stream, len, &mut tracker);
        quiet!("Data sent");

    }
    pub fn recv(mut stream: TcpStream, settings: Settings) {
        // Getting metadata
        let path = String::from_binary(&mut stream).unwrap();
        let len = u64::from_binary(&mut stream).unwrap();
        if !settings.auto_accept {
            if "n" == <Input>::yn().msg(&format!("Do you want to accept {path}?y/n")).get().unwrap() {
                return;
            }
        }
        quiet!("Recieving {path}");
        let file = std::fs::File::create(path).unwrap();
        let mut tracker = Tracker {
            written: 0,
            total_written: 0,
            previous: None
        };
        transfer(stream, file, len, &mut tracker);
        quiet!("Recieved");
    }
}
