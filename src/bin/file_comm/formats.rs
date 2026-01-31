use crate::Settings;
use abes_nice_things::{FromBinary, ToBinary};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
pub const CURRENT: u64 = 0;
// Client suggest format: u64
// Server counter offer None is refuse: Option<u64>
// Client accept/deny: bool
pub fn client_handshake(
    settings: &Settings,
    stream: &mut std::net::TcpStream,
) -> Result<u64, std::io::Error> {
    CURRENT.to_binary(stream).unwrap();
    let counter_offer = match <Option<u64>>::from_binary(stream).unwrap() {
        Some(format) => format,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to agree on format",
            ));
        }
    };
    if let Some(bound) = settings.forbid_lower
        && counter_offer < bound
    {
        false.to_binary(stream).unwrap();
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to agree on format",
        ))
    } else {
        true.to_binary(stream).unwrap();
        Ok(counter_offer)
    }
}
pub fn host_handshake(
    settings: &Settings,
    stream: &mut std::net::TcpStream,
) -> Result<u64, std::io::Error> {
    let offer = u64::from_binary(stream)?;
    let counter_offer = offer.min(CURRENT);
    if let Some(bound) = settings.forbid_lower
        && counter_offer < bound
    {
        None
    } else {
        Some(counter_offer)
    }
    .as_ref()
    .to_binary(stream)?;

    match bool::from_binary(stream)? {
        true => Ok(counter_offer),
        false => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to agree on format",
        )),
    }
}

pub fn send(
    settings: &Settings,
    stream: impl Into<BufWriter<TcpStream>>,
    path: String,
    format: u64,
    is_first: bool,
) -> std::io::Result<()> {
    match format {
        0 => format0::send(settings, stream.into(), path, is_first),
        x => unreachable!("Invalid format: {x}"),
    }
}
pub fn recv(settings: &Settings, stream: BufReader<TcpStream>, format: u64) -> std::io::Result<()> {
    match format {
        0 => format0::recv(settings, stream),
        x => unreachable!("Invalid format: {x}"),
    }
}
mod format0 {
    use super::*;
    use abes_nice_things::{FromBinary, MaxVec, ProgressBar, Style, ToBinary};
    use std::{io::Write, os::unix::fs::PermissionsExt};
    const BAR: ProgressBar<u64> = *ProgressBar::new(0, 0, 50)
        .eta(true)
        .supplementary_newline(true)
        .amount_done(true)
        .done_style(*Style::new().green().intense(true));
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    struct Flags(u8);
    impl Flags {
        const EXECUTE: u8 = 0b0000_0001;
        const ZIP: u8 = 0b0000_0010;
        fn validate(self) -> bool {
            // If it is a zip, it is not an executable file
            if self.zip() && self.execute() {
                return false;
            }
            true
        }
        fn new() -> Flags {
            Flags(0b0000_0000)
        }
        fn set_execute(&mut self) {
            self.0 |= Self::EXECUTE
        }
        fn set_zip(&mut self) {
            self.0 |= Self::ZIP
        }
        fn execute(self) -> bool {
            (self.0 & Self::EXECUTE) != 0
        }
        fn zip(self) -> bool {
            (self.0 & Self::ZIP) != 0
        }
    }
    impl FromBinary for Flags {
        fn from_binary(binary: &mut dyn std::io::Read) -> Result<Self, std::io::Error>
        where
            Self: Sized,
        {
            Ok(Flags(u8::from_binary(binary)?))
        }
    }
    impl ToBinary for Flags {
        fn to_binary(&self, binary: &mut dyn Write) -> Result<(), std::io::Error> {
            self.0.to_binary(binary)
        }
    }
    // Data:
    //  1) path: String
    //  2) data_len: u64
    //  3) flags: Flags
    //  4) get bool from recv for if we should go ahead with it
    //  5) data: &[u8]
    pub fn send(
        _settings: &Settings,
        mut stream: BufWriter<TcpStream>,
        mut path: String,
        is_first: bool,
    ) -> std::io::Result<()> {
        // Sending metadata
        path.to_binary(&mut stream)?;
        let mut flags = Flags::new();

        // Zip handling
        if std::fs::metadata(&path)?.is_dir() {
            // zip time!
            flags.set_zip();
            if !is_first && std::fs::exists(path.clone() + ".zip")? {
                // This isn't the first send of the zip and there is a zip which seems like it was
                // created by us previously
                if !std::process::Command::new("zip")
                    .arg(&path)
                    .spawn()?
                    .wait()?
                    .success()
                {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to create zip",
                    ));
                }
                path = (path + ".zip").to_string();
            }
        }
        let mut file = std::fs::File::open(path)?;
        let metadata = file.metadata()?;
        let len = metadata.len();
        len.to_binary(&mut stream)?;
        if !flags.zip() && (metadata.permissions().mode() & 0b001_001_001) != 0 {
            flags.set_execute();
        }
        if !flags.validate() {
            // If we reached this, it is not a user error, it is a coding error
            unreachable!("Attempted to send invalid flags: {:b}", flags.0)
        }
        flags.to_binary(&mut stream)?;
        stream.flush()?;

        if !bool::from_binary(stream.get_mut())? {
            eprintln!("recv decided not to");
            return Ok(());
        }

        // Setting up the progress bar
        #[allow(const_item_mutation)] // Not actually mutating it, it is just warning me that I am
        // working on a copy, which I know already
        let bar = (*BAR.target(len)).auto_update(std::time::Duration::from_millis(250));

        // Sending the data
        let mut sent = 0;
        let mut buf: MaxVec<u8, 1024> = MaxVec::new();
        while sent < len {
            buf.read_from(&mut file)?;
            stream.write_all(buf.as_slice())?;
            sent += buf.len() as u64;
            buf.empty_iffy();
            bar.set(sent);
        }
        bar.finish().unwrap().clear();
        stream.flush()
    }
    pub fn recv(settings: &Settings, mut stream: BufReader<TcpStream>) -> std::io::Result<()> {
        // Getting metadata
        let path = String::from_binary(&mut stream)?;
        let len = u64::from_binary(&mut stream)?;
        let flags = Flags::from_binary(&mut stream)?;

        // Creating file
        if !settings.replace_if_needed
            && std::fs::exists(&path)?
            && !<abes_nice_things::Input>::yn()
                .msg(format!(
                    "{path} already exists, \
                    are you sure you want to overwrite it? y/n"
                ))
                .get()
        {
            // Decided not to overwrite
            false.to_binary(stream.get_mut())?;
            return Ok(());
        } else {
            true.to_binary(stream.get_mut())?;
        }
        let mut file = std::fs::File::create(&path)?;
        if flags.execute()
            && !std::process::Command::new("chmod")
                .arg("+x")
                .arg(&path)
                .status()?
                .success()
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to change file permissions",
            ));
        }

        // Recieving data
        let mut buf: MaxVec<u8, 1024> = MaxVec::new();
        let mut recieved = 0;
        #[allow(const_item_mutation)] // see above
        let bar = (*BAR.target(len)).auto_update(std::time::Duration::from_millis(250));
        while recieved < len {
            buf.read_from(&mut stream)?;
            file.write_all(buf.as_slice())?;
            recieved += buf.len() as u64;
            buf.empty_iffy();
            bar.set(recieved);
        }

        bar.finish().unwrap().clear();

        if flags.zip()
            && !std::process::Command::new("unzip")
                .arg(path)
                .spawn()?
                .wait()?
                .success()
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to unzip directory",
            ));
        }

        Ok(())
    }
}
