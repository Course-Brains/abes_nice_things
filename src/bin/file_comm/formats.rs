use crate::{quiet, Mode, Settings};
use abes_nice_things::{FromBinary, ToBinary};
use std::net::TcpStream;

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
        Mode::Send => send_hand_shake(stream, settings),
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
        _ => return Err("Invalid format given by other".to_string()),
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
        _ => unreachable!(),
    };
    Ok(())
}
mod f0 {
    // Format:
    // Length of name(u32)
    // name
    // data
    use crate::{quiet, Settings};
    use abes_nice_things::{input, FromBinary, Input, ToBinary};
    use std::{
        fs::File,
        io::{Read, Write},
        net::TcpStream,
        path::PathBuf,
    };

    static TO_SEND: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);
    pub fn send(mut stream: TcpStream, settings: Settings) {
        let file;
        let path = match TO_SEND.try_lock().unwrap().clone() {
            Some(path) => {
                file = File::open(&path).unwrap();
                path
            }
            None => loop {
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
                            if settings.path.is_some() {
                            } else if *"y"
                                == <Input>::yn()
                                    .msg("Do you want to use this for subsequent requests?y/n")
                                    .get()
                                    .unwrap()
                            {
                                *TO_SEND.try_lock().unwrap() = Some(PathBuf::from(&path))
                            }
                        }
                        break PathBuf::from(path);
                    }
                    Err(error) => eprintln!("Failed to identify file validity: {error}"),
                }
            },
        };
        let path = path
            .file_name()
            .expect("Failed to get file name")
            .to_str()
            .unwrap();
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
        if settings.auto_accept {
        } else if "n"
            == match stream.peer_addr() {
                Ok(addr) => <Input>::yn()
                    .msg(&format!(
                        "Are you sure you want to accept {name} from {addr}?y/n"
                    ))
                    .get()
                    .unwrap(),
                Err(_) => <Input>::yn()
                    .msg(&format!(
                        "Are you sure you want to accept {name} from unknown?y/n"
                    ))
                    .get()
                    .unwrap(),
            }
        {
            return;
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
    use crate::{quiet, Settings, QUIET};
    use abes_nice_things::{
        input, progress_bar::Rate, FromBinary, Input, ProgressBar, Style, ToBinary,
    };
    use std::io::{Read, Write};
    use std::net::TcpStream;
    // Assumed to be less than 2^32
    const BUFFER_SIZE: u64 = 1000;
    fn transfer(mut from: impl Read, mut to: impl Write, len: u64) {
        let progress_bar = match QUIET.load(std::sync::atomic::Ordering::Relaxed) {
            true => None,
            false => Some(
                *ProgressBar::new(0, len, 50)
                    .done_style(*Style::new().cyan().intense(true))
                    .waiting_style(*Style::new().red())
                    .supplementary_newline(true)
                    .percent_done(true)
                    .rate(Some(Rate::Bytes))
                    .eta(true),
            ),
        };
        let proxy = progress_bar.map(|mut bar| {
            bar.draw();
            bar.auto_update(std::time::Duration::from_millis(500))
        });
        let mut buf = const { [0_u8; BUFFER_SIZE as usize] };
        let mut remaining = 0;
        while remaining > BUFFER_SIZE {
            // First the buffer sized chunks
            from.read_exact(&mut buf).unwrap();
            to.write_all(&mut buf).unwrap();
            remaining -= BUFFER_SIZE;
            if let Some(proxy) = &proxy {
                proxy.set(len - remaining);
            }
        }
        // Then the remainder
        let mut buf = vec![0_u8; remaining as usize];
        from.read_exact(&mut buf).unwrap();
        to.write_all(&mut buf).unwrap();
        if let Some(proxy) = proxy {
            proxy.finish().unwrap().clear();
        }
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
                    Err(error) => eprintln!("Failed to determine if the file exists: {error}"),
                }
            }
        });
        if !settings.auto_accept
            && "n"
                == <Input>::yn()
                    .msg(&format!("Do you want to send {path}?y/n"))
                    .get()
                    .unwrap()
        {
            return;
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
        transfer(file, stream, len);
        quiet!("Data sent");
    }
    pub fn recv(mut stream: TcpStream, settings: Settings) {
        // Getting metadata
        let path = String::from_binary(&mut stream).unwrap();
        let len = u64::from_binary(&mut stream).unwrap();
        if !settings.auto_accept {
            if "n"
                == <Input>::yn()
                    .msg(&format!("Do you want to accept {path}?y/n"))
                    .get()
                    .unwrap()
            {
                return;
            }
        }
        quiet!("Recieving {path}");
        let file = std::fs::File::create(path).unwrap();
        transfer(stream, file, len);
        quiet!("Recieved");
    }
}
