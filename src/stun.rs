use crate::MaxVec;
use crate::ProgressBar;
use crate::from_binary::Result;
use crate::from_binary::err;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::net::UdpSocket;
use std::num::NonZeroU32;
use std::time::Duration;

const MAX_PACKET_SIZE: usize = 1028;
pub fn test() {
    let errors = std::sync::Mutex::new(std::collections::HashSet::new());
    let progress_bar = ProgressBar::new(0_usize, 272, 100)
        .supplementary_newline(true)
        .amount_done(true)
        .rate(Some(crate::progress_bar::Rate::Absolute))
        .eta(true)
        .timer(crate::progress_bar::Timer::Mean)
        .percent_done(true)
        .auto_update(std::time::Duration::from_secs(1));
    let servers = std::fs::read_to_string("stun_servers").unwrap();
    let servers = servers.split('\n').filter(|s| !s.is_empty());
    std::thread::scope(|spawner| {
        let mut handles = Vec::with_capacity(272);
        for server in servers {
            let server = server.to_string();
            let errors = &errors;
            let progress_bar = &progress_bar;
            handles.push(spawner.spawn(move || {
                match testing(server.as_str()) {
                    Ok(_) => panic!("WE HAD A SUCCESS WITH {server}!!!!!!!"),
                    Err(error) => {
                        std::fs::write(format!("logs/{server}"), error.to_string()).unwrap();
                        errors.lock().unwrap().insert(error.to_string());
                    }
                }
                progress_bar.add(1);
            }));
        }
        for handle in handles.into_iter() {
            handle.join().unwrap();
        }
    });
    progress_bar.finish().unwrap().clear();
    for error in errors.lock().unwrap().iter() {
        println!("{error}\n");
    }
}

fn testing(server: &str) -> Result<()> {
    let mut socket = UdpSocket::bind((std::net::Ipv4Addr::UNSPECIFIED, 0)).unwrap();
    socket.connect(server)?;
    //socket
    //    .connect(std::net::SocketAddr::from(([23, 21, 150, 121], 3478)))
    //    .unwrap();
    stun(&mut socket)
}

fn stun(socket: &mut UdpSocket) -> Result<()> {
    //println!("Starting STUN exchange");
    let header = Header::new(MessageClass::Request, MessageMethod::Binding, 0);
    let mut karn = KarnState::new();
    let response = exchange(socket, &mut karn, header, &[])?;
    let response_header = Header::from_data(*((*response)[0..20]).as_array().unwrap())?;
    Ok(())
}

/// Exchange two packets between this computer and the recipient using karn's algorithm to handle
/// retransmission and ensuring that the correct packet is returned
fn exchange(
    socket: &mut UdpSocket,
    karn: &mut KarnState,
    header: Header,
    data: &[u8],
) -> Result<MaxVec<u8, MAX_PACKET_SIZE>> {
    let mut packet = Vec::from(header.assemble());
    packet.extend_from_slice(data);
    if packet.len() > MAX_PACKET_SIZE {
        return err(
            ErrorKind::InvalidInput,
            format!(
                "Attempted to send packet above maximum size\n{} > {MAX_PACKET_SIZE}",
                packet.len()
            ),
        );
    }
    karn.log_new_packet();
    let start = std::time::Instant::now();
    'back_off: while let Some(timeout) = karn.get_timeout() {
        socket.set_read_timeout(Some(timeout))?;
        socket.send(data)?;
        let buf = loop {
            let mut buf = MaxVec::new();
            assert!(buf.is_empty());
            let packet_start = std::time::Instant::now();
            // SAFETY: We are setting the len properly
            let status = socket.recv(unsafe { buf.as_unfilled_slice_mut() });
            let packet_elapsed = packet_start.elapsed();
            match status {
                Ok(written) => unsafe { buf.set_len(written) },
                Err(error) => {
                    if error.kind() == ErrorKind::TimedOut || error.kind() == ErrorKind::WouldBlock
                    {
                        karn.log_retransmit();
                        continue 'back_off;
                    }
                }
            }
            if buf.len() < 20 {
                return err(ErrorKind::InvalidData, "Invalid header(too small)");
            }
            if let Ok(packet_header) = Header::from_data(*((*buf)[0..20]).as_array().unwrap())
                && packet_header.transaction_id != header.transaction_id
            {
            } else {
                eprintln!("recieved wrong packet");
                // Wrong packet, continue reading
                socket.set_read_timeout(Some(timeout.saturating_sub(packet_elapsed)))?;
                continue;
            }
            break buf;
        };
        // If we get here then the read succeeded
        karn.log_received(start.elapsed());
        return Ok(buf);
    }

    err(ErrorKind::TimedOut, "Exceded maximum retries allowed")
}

struct KarnState {
    round_trip_time_estimate: Duration,
    back_off_mult: NonZeroU32,
    retransmit_count: u16,
}
impl KarnState {
    const ALPHA: f32 = 0.875;
    const BETA: f32 = 2.0;
    const TIMEOUT_THRESHOLD: u16 = 7;
    fn new() -> KarnState {
        KarnState {
            // Default timeout of 1 second as described by RFC-6298
            round_trip_time_estimate: Duration::from_millis(500),
            back_off_mult: NonZeroU32::new(1).unwrap(),
            retransmit_count: 0,
        }
    }
    fn get_timeout(&self) -> Option<Duration> {
        // Times out after 32 consequtive failures
        if self.retransmit_count < Self::TIMEOUT_THRESHOLD {
            // Rounding up to 1 second as described in RFC-6298
            // Maximum of one minute as also described
            Some(
                (self.round_trip_time_estimate.mul_f32(Self::BETA) * self.back_off_mult.get())
                    .max(Duration::from_secs(1))
                    .min(Duration::from_secs(60)),
            )
        } else {
            None
        }
    }
    fn log_retransmit(&mut self) {
        self.retransmit_count += 1;
        // Doubling according to RFC-8489
        self.back_off_mult = self
            .back_off_mult
            .saturating_mul(NonZeroU32::new(2).unwrap());
    }
    fn log_received(&mut self, sample: Duration) {
        // We actually update the estimate
        if self.retransmit_count == 0 {
            self.round_trip_time_estimate = self.round_trip_time_estimate.mul_f32(Self::ALPHA)
                + sample.mul_f32(1.0 - Self::ALPHA);
            self.back_off_mult = NonZeroU32::new(1).unwrap();
        }
        self.retransmit_count = 0;
    }
    fn log_new_packet(&mut self) {
        self.retransmit_count = 0;
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Header {
    class: MessageClass,
    method: MessageMethod,
    message_len: u16,
    transaction_id: [u8; 12],
}
impl Header {
    const MAGIC_COOKIE: [u8; 4] = [0x21, 0x12, 0xA4, 0x42];
    fn new(class: MessageClass, method: MessageMethod, message_len: u16) -> Header {
        let mut id = [0; 12];
        crate::random::RANDOM
            .with_borrow_mut(|random| random.read_exact(&mut id))
            .unwrap();
        Header {
            class,
            method,
            message_len,
            transaction_id: id,
        }
    }
    fn assemble(&self) -> [u8; 20] {
        // first 16 are made up of the message type
        let mut out = [0; 20];

        let method = self.method.get_bytes();
        out[0] = method[0];
        out[1] = method[1];
        // The last byte of byte 1 is the most significant bit of the class
        out[0] |= self.class.most_significant();
        // The fourth bit of byte 2 is the least significant of the class
        out[1] |= self.class.least_significant() << 4;

        // Next two are the message length
        let len = self.message_len.to_be_bytes();
        out[2] = len[0];
        out[3] = len[1];

        // Next four are the cookie
        let cookie = Header::MAGIC_COOKIE;
        for i in 0..4 {
            out[i + 4] = cookie[i];
        }

        // Last 12 are the transaction id
        for i in 0..12 {
            out[i + 8] = self.transaction_id[i]
        }

        out
    }
    fn from_data(data: [u8; 20]) -> Result<Header> {
        if (data[0] & 0b1100_0000) != 0 {
            return err(
                ErrorKind::InvalidData,
                "Invalid header: First two bits were not zero",
            );
        }

        // Now for hell
        let mut class_byte = (data[0] & 0b0000_0001) << 1;
        class_byte |= (data[1] & 0b0001_0000) >> 4;
        let class = MessageClass::from_bits(class_byte)?;
        // hell part 2
        let mut method_bytes = [0, data[1] & 0b0000_1111];
        method_bytes[1] |= (data[1] & 0b1110_0000) >> 1;
        method_bytes[1] |= (data[0] & 0b0000_0010) << 6;
        method_bytes[0] |= (data[0] & 0b0011_1100) >> 2;
        let method = MessageMethod::from_bytes(method_bytes)?;

        let len = u16::from_be_bytes([data[2], data[3]]);

        if data[4..8] != Header::MAGIC_COOKIE {
            return err(
                ErrorKind::InvalidData,
                "Invalid header: magic cookie was incorrect",
            );
        }

        let transaction_id = *data[8..20].as_array().unwrap();

        Ok(Header {
            class,
            method,
            message_len: len,
            transaction_id,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
enum MessageClass {
    Request = 0b00,
    Indication = 0b01,
    Success = 0b10,
    Error = 0b11,
}
impl MessageClass {
    fn most_significant(self) -> u8 {
        self.to_u8() >> 1
    }
    fn least_significant(self) -> u8 {
        self.to_u8() & 0b1
    }
    const fn to_u8(self) -> u8 {
        unsafe { std::mem::transmute(self) }
    }
    fn from_bits(bits: u8) -> Result<Self> {
        Ok(match bits {
            0b00 => Self::Request,
            0b01 => Self::Indication,
            0b10 => Self::Success,
            0b11 => Self::Error,
            other => {
                return err(
                    ErrorKind::InvalidData,
                    format!("Got invalid message class: {other:b}"),
                );
            }
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u16)]
enum MessageMethod {
    Binding = 0x001,
}
impl MessageMethod {
    // |------|------------------|
    // B A 9 8 7   6 5 4   3 2 1 0
    //           ^C1     ^C0
    // |--------------|--------------|
    // X X B A 9 8 7 C 6 5 4 C 3 2 1 0
    fn get_bytes(self) -> [u8; 2] {
        debug_assert_eq!(self.get_raw() & 0b1111_0000_0000_0000, 0);
        let mut out = [0; 2];

        // first 2 bits are always 0 and last bit of first byte is for class
        let first_section = (self.get_raw() >> 6).to_be_bytes()[1] & 0b0011_1110;
        out[0] |= first_section;

        // 1st to 3rd bits of second byte are 2nd - 4th of raw
        let second_section = (self.get_raw().to_be_bytes()[1] & 0b0111_0000) << 1;
        out[1] |= second_section;

        // 5th through 8th of second byte are the same as input
        let third_section = self.get_raw().to_be_bytes()[1] & 0b0000_1111;
        out[1] |= third_section;

        out
    }
    fn get_raw(self) -> u16 {
        unsafe { std::mem::transmute(self) }
    }
    fn from_bytes(bytes: [u8; 2]) -> Result<MessageMethod> {
        let value = u16::from_be_bytes(bytes);
        Ok(match value {
            0x001 => Self::Binding,
            other => {
                return err(
                    ErrorKind::InvalidData,
                    format!("Invalid message method: {other:x}"),
                );
            }
        })
    }
}

enum Attribute {
    MappedAddress {
        family: attributes::MappedAddressFamily,
        port: u16,
    },
    Username,
    MessageIntegrity,
    ErrorCode {
        // The hundreds place
        code_class: u8,
        // the rest of the number
        code: u8,
        reason: String,
    },
    UnknownAttributes,
    Realm,
    Nonce,
    XorMappedAddress {
        family: attributes::MappedAddressFamily,
        port: u16,
    },
}
impl Attribute {
    fn to_bytes(&self, header: &Header) -> Result<Vec<u8>> {
        let attribute_type: u16 = match self {
            Self::MappedAddress { .. } => 0x0001,
            Self::Username => 0x0006,
            Self::MessageIntegrity => 0x0008,
            Self::ErrorCode { .. } => 0x0009,
            Self::UnknownAttributes => 0x000A,
            Self::Realm => 0x0014,
            Self::Nonce => 0x0015,
            Self::XorMappedAddress { .. } => 0x0020,
        };
        let mut out = Vec::from(attribute_type.to_be_bytes());

        // We put a placeholder for the length
        out.push(0);
        out.push(0);

        // Now we put the actual data
        match self {
            Self::MappedAddress { family, port } => {
                // first byte is 0
                out.push(0);
                // second is the family
                out.push(family.get_family());
                // third and fourth are the port
                let port = port.to_be_bytes();
                out.push(port[0]);
                out.push(port[1]);
                // with the address slapped on the back
                out.extend_from_slice(family.get_data());
            }
            Self::ErrorCode {
                code_class,
                code,
                reason,
            } => {
                // first two are zero
                out.push(0);
                out.push(0);
                // third is the class from 3-6
                if *code_class > 6 || *code_class < 3 {
                    return err(
                        ErrorKind::InvalidInput,
                        "Attempted to create invalid error code\n\
                    Attempted to create code with class {code_class} when only allowed from 3-6",
                    );
                }
                out.push(*code_class);
                // fourth is the remaining piece of the code
                if *code > 99 {
                    return err(
                        ErrorKind::InvalidInput,
                        "Attempted to create invalid error code\n\
                    Attempted to create code with modulus {code} when it must be <= 99",
                    );
                }
                out.push(*code);
                // The rest are the reason as UTF-8 encoded bytes with zeros at the end
                out.extend_from_slice(reason.as_bytes());
                for _ in 0..(4_usize.saturating_sub(out.len() % 4)) {
                    out.push(0);
                }
            }
            Self::XorMappedAddress { family, port } => {
                //first byte is 0
                out.push(0);
                // second is the famil
                out.push(family.get_family());
                // next two are the xport which is made by xoring the port with the first two bytes
                // of the magic cookie
                let port = port.to_be_bytes();
                out.push(port[0] ^ Header::MAGIC_COOKIE[0]);
                out.push(port[1] ^ Header::MAGIC_COOKIE[1]);
                // remaining are the xaddress which is address xored with the concatenation of the
                // magic cookie and the transaction id
                let data = family.get_data();
                for i in 0..4 {
                    out.push(data[i] ^ Header::MAGIC_COOKIE[i]);
                }
                if data.len() == 16 {
                    for i in 0..12 {
                        out.push(data[i + 4] ^ header.transaction_id[i]);
                    }
                }
            }
            _ => {}
        }

        // Replace the placeholder length
        let len = (out.len() as u16 - 4).to_be_bytes();
        out[2] = len[0];
        out[3] = len[1];

        assert_eq!(out.len() % 4, 0);

        Ok(out)
    }
}
mod attributes {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub enum MappedAddressFamily {
        IPv4([u8; 4]),
        IPv6([u8; 16]),
    }
    impl MappedAddressFamily {
        pub fn get_family(&self) -> u8 {
            match self {
                Self::IPv4(_) => 0x01,
                Self::IPv6(_) => 0x02,
            }
        }
        pub fn get_data(&self) -> &[u8] {
            match &self {
                Self::IPv4(data) => data,
                Self::IPv6(data) => data,
            }
        }
    }
}
