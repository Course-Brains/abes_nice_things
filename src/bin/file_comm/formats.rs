use std::net::TcpStream;
use abes_nice_things::{ToBinary, FromBinary};
use crate::{Settings, Mode, quiet};

pub type FormatID = u32;
pub const HIGHEST: FormatID = 0;

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
    settings.get_format().to_binary(&mut stream);
    // Format is decided by the reciever
    let format = FormatID::from_binary(&mut stream);
    quiet!("Decided to use format: {format}\nFormat handshake done");
    match format {
        0 => f0::send(stream, settings),
        _ => return Err("Invalid format given by other".to_string())
    }
    Ok(())
}
fn recv_hand_shake(mut stream: TcpStream, settings: Settings) -> Result<(), String> {
    quiet!("Waiting for suggested format");
    let other_highest = FormatID::from_binary(&mut stream);
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
    Ok::<u32, String>(format).to_binary(&mut stream);
    quiet!("Format handshake done");
    match format {
        0 => f0::recv(stream, settings),
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
        (path.len() as u32).to_binary(&mut stream);
        stream.write_all(path.as_bytes()).unwrap();
        quiet!("Sending file");
        transfer(file, stream, len, 1000);
        quiet!("File sent")
    }
    pub fn recv(mut stream: TcpStream, settings: Settings) {
        quiet!("Getting metadata");
        let name_len = u32::from_binary(&mut stream);
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
    fn transfer(mut from: impl Read, mut to: impl Write, mut len: u64, interval: usize) {
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
