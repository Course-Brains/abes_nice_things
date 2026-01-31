use abes_nice_things::Input;
use std::io::{BufReader, BufWriter};
mod formats;
fn main() {
    let settings = Settings::new();
    match settings.role {
        Role::Host(port) => host(&settings, port),
        Role::Client(ref host) => client(&settings, host),
    }
}
fn host(settings: &Settings, port: u16) {
    let mut is_first = true;
    for stream in std::net::TcpListener::bind((std::net::Ipv4Addr::UNSPECIFIED, port))
        .unwrap()
        .incoming()
    {
        if let Ok(mut stream) = stream {
            if let Ok(format) = formats::host_handshake(settings, &mut stream) {
                let _ = match settings.mode {
                    Mode::Send(ref path) => formats::send(
                        settings,
                        BufWriter::new(stream),
                        path.clone(),
                        format,
                        is_first,
                    ),
                    Mode::Recv => formats::recv(settings, BufReader::new(stream), format),
                };
                is_first = false;
            }
        }
    }
}
fn client(settings: &Settings, host: &String) {
    let mut stream = std::net::TcpStream::connect(host).unwrap();
    let format = formats::client_handshake(settings, &mut stream)
        .expect("Failed to agree on format with host");
    match settings.mode {
        Mode::Send(ref path) => {
            formats::send(settings, BufWriter::new(stream), path.clone(), format, true)
        }
        Mode::Recv => formats::recv(settings, BufReader::new(stream), format),
    }
    .unwrap();
}
struct Settings {
    mode: Mode,
    role: Role,
    forbid_lower: Option<u64>,
    replace_if_needed: bool,
}
impl Settings {
    fn new() -> Settings {
        let mut mode = None;
        let mut role = None;
        let mut forbid_lower = None;
        let mut replace_if_needed = false;

        let mut args = std::env::args();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--send" | "-s" => {
                    mode = Some(Mode::Send(
                        args.next().expect("File path needed after --send"),
                    ))
                }
                "--recv" | "-r" => mode = Some(Mode::Recv),
                "--host" => {
                    role = Some(Role::Host(
                        args.next()
                            .expect("Port must be placed after --host")
                            .parse()
                            .unwrap(),
                    ))
                }
                "--client" => {
                    role = Some(Role::Client(args.next().expect("Need host after --client")));
                }
                "--forbid_lower" => forbid_lower = Some(args.next().unwrap().parse().unwrap()),
                "--replace-if-needed" => replace_if_needed = true,
                "help" => println!("{}", include_str!("help.txt")),
                _ => {}
            }
        }

        if let Some(forbid_lower) = forbid_lower
            && forbid_lower > formats::CURRENT
        {
            panic!(
                "Attempted to forbid all valid formats: attempted to forbid lower than\
                {forbid_lower} when the highest available format is {}",
                formats::CURRENT
            );
        }
        if mode.is_none() {
            let choice = <Input>::allow(vec!["send", "s", "recv", "r"])
                .msg("Send or recv a file?")
                .get();
            match choice.as_str() {
                "send" | "s" => {
                    println!("What file?");
                    mode = Some(Mode::Send(abes_nice_things::input()));
                }
                "recv" | "r" => mode = Some(Mode::Recv),
                _ => unreachable!("Fucko boingo"),
            }
        }
        if role.is_none() {
            role = Some(Role::Client(
                <Input>::new().msg("host ip:port to connect to").get(),
            ));
        }

        Settings {
            mode: mode.unwrap(),
            role: role.unwrap(),
            forbid_lower,
            replace_if_needed,
        }
    }
}
#[derive(Clone)]
enum Mode {
    Send(String),
    Recv,
}
enum Role {
    Host(u16),
    Client(String),
}
