use crate::MaxVec;
use std::io::Read;
use std::io::Write;

pub enum FlushSource {
    Zeros,
    Reader(Box<dyn Read>),
    #[cfg(unix)]
    Random,
    #[cfg(unix)]
    URandom,
}

pub struct DecryptReader<R: Read> {
    round_keys: [Block; 15],
    /// The output buffer for caching the output when not all of it is needed at once
    buffer: [u8; 16],
    /// How far through the buffer we are, so if this is 5 then the remaining data in the buffer is
    /// from index 5 to index 15
    buffer_progress: u8,
    inner: R,
    prev_block: Block,
}
impl<R: Read> DecryptReader<R> {
    pub fn new(inner: R, initialization_vector: [u8; 16], key: &[u8]) -> Option<DecryptReader<R>> {
        Some(Self {
            round_keys: key_expansion(key)?,
            buffer: [0; 16],
            buffer_progress: 16,
            inner,
            prev_block: Block::from_bytes(initialization_vector),
        })
    }
    /// Empties the output buffer, this is helpful because it will correspond to each flush of the
    /// writer.
    ///
    /// Flushing the writer with a non-full buffer will fill the remainder with dummy data, this
    /// will do the inverse and if you line up the reads, writes, flush and dump then you can flush
    /// the data without reading any dummy data on this side.
    ///
    /// This will return the amount of data skipped in case you want to put that in an assert
    pub fn dump_buffer(&mut self) -> u8 {
        let skipped = self.buffer.len() as u8 - self.buffer_progress;
        self.buffer_progress = 0;
        skipped
    }
    pub fn to_inner(self) -> R {
        self.inner
    }
    pub const fn get_ref(&self) -> &R {
        &self.inner
    }
    pub const fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }
}
impl<R: Read> Read for DecryptReader<R> {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        // If we have data in the output buffer then let's use that
        if self.buffer_progress < self.buffer.len() as u8 {
            let amount = buf.write(&self.buffer[self.buffer_progress as usize..])?;
            self.buffer_progress += amount as u8;
            return Ok(amount);
        }

        if let Err(error) = self.inner.read_exact(&mut self.buffer) {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                return Ok(0);
            } else {
                return Err(error);
            }
        }
        // buffer has crypt
        let prev = self.prev_block;
        self.prev_block = Block::from_bytes(self.buffer);
        // prev(_block) has crypt
        let mut decrypted = Block::from_bytes(self.buffer).inv_cipher(&self.round_keys);
        decrypted ^= prev;
        self.buffer = decrypted.to_bytes();
        self.buffer_progress = 0;

        let amount = buf.write(&self.buffer)?;
        self.buffer_progress += amount as u8;
        Ok(amount)
    }
}
pub struct EncryptWriter<W: Write> {
    round_keys: [Block; 15],
    buffer: MaxVec<u8, 16>,
    inner: W,
    prev_block: Block,
    flush_source: FlushSource,
}
impl<W: Write> EncryptWriter<W> {
    pub fn new(
        inner: W,
        initialization_vector: [u8; 16],
        key: &[u8],
        flush_source: FlushSource,
    ) -> Option<Self> {
        Some(Self {
            round_keys: key_expansion(key)?,
            buffer: MaxVec::new(),
            inner,
            prev_block: Block::from_bytes(initialization_vector),
            flush_source,
        })
    }
    pub const fn get_ref(&self) -> &W {
        &self.inner
    }
    pub const fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
}
impl<W: Write> Write for EncryptWriter<W> {
    fn write(&mut self, mut buf: &[u8]) -> std::io::Result<usize> {
        // See if we can skip the buffer
        if self.buffer.is_empty() && buf.len() >= 16 {
            let mut data = [0; 16];
            buf.read_exact(&mut data).unwrap(); // infallible
            let mut plain = Block::from_bytes(data);
            plain ^= self.prev_block;
            let encrypted = plain.cipher(&self.round_keys);
            self.prev_block = encrypted;
            self.inner.write_all(&encrypted.to_bytes())?;
            return Ok(16);
        }

        let amount_read = self.buffer.read_from(&mut buf)?;
        // If the buffer is full then we send it off
        if self.buffer.is_full() {
            let mut plain = Block::from_bytes(self.buffer.take_if_full().unwrap()); // infallible
            plain ^= self.prev_block;

            let encrypted = plain.cipher(&self.round_keys);
            self.prev_block = encrypted;
            // yucky
            self.inner.write_all(&encrypted.to_bytes())?;
        }

        // This is correct because we are writing it to the internal buffer
        Ok(amount_read)
    }
    // When flushing we will fill the required space with 0s before encrypting
    fn flush(&mut self) -> std::io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        if !self.buffer.is_full() {
            match &mut self.flush_source {
                FlushSource::Zeros => self
                    .buffer
                    .read_from_exact(&mut std::io::repeat(0))
                    .unwrap(), // infallible
                FlushSource::Reader(reader) => self.buffer.read_from_exact(reader)?,
                #[cfg(unix)]
                FlushSource::Random => crate::random::RANDOM
                    .with_borrow_mut(|random| self.buffer.read_from_exact(random))?,
                #[cfg(unix)]
                FlushSource::URandom => crate::random::URANDOM
                    .with_borrow_mut(|urandom| self.buffer.read_from_exact(urandom))?,
            }
        }
        let plain = self.buffer.take_if_full().unwrap(); // infallible
        let encrypted = (Block::from_bytes(plain) ^ self.prev_block)
            .cipher(&self.round_keys)
            .to_bytes();
        self.prev_block = Block::from_bytes(encrypted);
        // yucky 2: electric boogaloo
        self.inner.write_all(&encrypted)?;
        self.inner.flush()
    }
}
impl<W: Write> Drop for EncryptWriter<W> {
    fn drop(&mut self) {
        // I'm not about to panic here
        let _ = self.flush();
    }
}

// We are using 256 bit keys so there are 14 rounds
const NUM_ROUNDS: usize = 14;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
struct Byte(u8);
impl Byte {
    fn double(self) -> Byte {
        Byte(match (self.0 & 0b1000_0000) != 0 {
            true => (self.0 << 1) ^ 0b0001_1011,
            false => self.0 << 1,
        })
    }
    fn s_box(self) -> Byte {
        S_BOX[self.0 as usize]
    }
    fn s_box_assign(&mut self) -> &mut Self {
        *self = self.s_box();
        self
    }
    fn inv_s_box(self) -> Byte {
        INV_S_BOX[self.0 as usize]
    }
    fn inv_s_box_assign(&mut self) -> &mut Self {
        *self = self.inv_s_box();
        self
    }
}
// Addition is defined as xor
impl std::ops::Add for Byte {
    type Output = Byte;
    fn add(self, rhs: Self) -> Self::Output {
        Byte(self.0 ^ rhs.0)
    }
}
impl std::ops::AddAssign for Byte {
    fn add_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}
// Repeated doubling for convinience
impl std::ops::Shl<usize> for Byte {
    type Output = Byte;
    fn shl(mut self, rhs: usize) -> Self::Output {
        for _ in 0..rhs {
            self = self.double()
        }
        self
    }
}
impl std::ops::Mul for Byte {
    type Output = Byte;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut out = Byte(0);

        for bit in 0..8 {
            if (self.0 & (0b1 << bit)) != 0 {
                out += rhs << bit;
            }
        }

        out
    }
}
impl std::ops::Mul<u8> for Byte {
    type Output = Byte;
    fn mul(self, rhs: u8) -> Self::Output {
        self * Byte(rhs)
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Word([Byte; 4]);
impl Word {
    fn mix_columns(self) -> Word {
        let mut out = [Byte(0); 4];
        let this = self.0;
        out[0] = this[0].double() + (this[1] * 3) + this[2] + this[3];
        out[1] = this[0] + this[1].double() + (this[2] * 3) + this[3];
        out[2] = this[0] + this[1] + this[2].double() + (this[3] * 3);
        out[3] = (this[0] * 3) + this[1] + this[2] + (this[3].double());
        Word(out)
    }
    fn inv_mix_columns(self) -> Word {
        let mut out = [Byte(0); 4];
        let this = self.0;
        out[0] = (this[0] * 0x0e) + (this[1] * 0x0b) + (this[2] * 0x0d) + (this[3] * 0x09);
        out[1] = (this[0] * 0x09) + (this[1] * 0x0e) + (this[2] * 0x0b) + (this[3] * 0x0d);
        out[2] = (this[0] * 0x0d) + (this[1] * 0x09) + (this[2] * 0x0e) + (this[3] * 0x0b);
        out[3] = (this[0] * 0x0b) + (this[1] * 0x0d) + (this[2] * 0x09) + (this[3] * 0x0e);
        Word(out)
    }
    fn sub_word(self) -> Word {
        Word([
            self.0[0].s_box(),
            self.0[1].s_box(),
            self.0[2].s_box(),
            self.0[3].s_box(),
        ])
    }
    fn rot_word(self) -> Word {
        Word([self.0[1], self.0[2], self.0[3], self.0[0]])
    }
}
impl std::ops::Add for Word {
    type Output = Word;
    fn add(self, rhs: Self) -> Self::Output {
        Word([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
        ])
    }
}
impl std::ops::AddAssign for Word {
    fn add_assign(&mut self, rhs: Self) {
        self.0[0] += rhs.0[0];
        self.0[1] += rhs.0[1];
        self.0[2] += rhs.0[2];
        self.0[3] += rhs.0[3];
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct Block([Word; 4]);
impl Block {
    fn from_bytes(bytes: [u8; 16]) -> Block {
        unsafe { std::mem::transmute(bytes) }
    }
    fn to_bytes(self) -> [u8; 16] {
        unsafe { std::mem::transmute(self) }
    }
    fn get_column(self, column: usize) -> Word {
        self.0[column]
    }
    fn get_column_mut(&mut self, column: usize) -> &mut Word {
        &mut self.0[column]
    }
    fn sub_bytes(&mut self) -> &mut Self {
        for word in self.0.iter_mut() {
            for byte in word.0.iter_mut() {
                byte.s_box_assign();
            }
        }
        self
    }
    fn inv_sub_bytes(&mut self) -> &mut Self {
        for word in self.0.iter_mut() {
            for byte in word.0.iter_mut() {
                byte.inv_s_box_assign();
            }
        }
        self
    }
    fn shift_rows(&mut self) -> &mut Self {
        // Nothing happens to the top row
        let buffer = &mut Byte(0);

        // Kind of neat that the solution has 101, 202 and 303 as the first moves in each row
        for (row, first_col, second_col) in [
            // We can do the second row by swapping adjacent as we go across
            (1, 0, 1),
            (1, 1, 2),
            (1, 2, 3),
            // We can do the third row by swapping the first and third then the second and fourth
            (2, 0, 2),
            (2, 1, 3),
            // We can do the fourth row by swapping the first and last then swapping adjacent from
            // the left stopping before we reach the first
            (3, 0, 3),
            (3, 2, 3),
            (3, 1, 2),
        ]
        .into_iter()
        {
            // l: l
            // r: r
            // _: _
            std::mem::swap(&mut self.get_column_mut(first_col).0[row], buffer);
            // l: _
            // r: r
            // _: l
            std::mem::swap(&mut self.get_column_mut(second_col).0[row], buffer);
            // l: _
            // r: l
            // _: r
            std::mem::swap(&mut self.get_column_mut(first_col).0[row], buffer);
            // l: r
            // r: l
            // _: _
        }
        self
    }
    fn inv_shift_rows(&mut self) -> &mut Self {
        let buffer = &mut Byte(0);
        for (row, first_col, second_col) in [
            // Literally just the reverse order of what is used in shift_rows
            (3, 1, 2),
            (3, 2, 3),
            (3, 0, 3),
            (2, 1, 3),
            (2, 0, 2),
            (1, 2, 3),
            (1, 1, 2),
            (1, 0, 1),
        ]
        .into_iter()
        {
            std::mem::swap(&mut self.get_column_mut(first_col).0[row], buffer);
            std::mem::swap(&mut self.get_column_mut(second_col).0[row], buffer);
            std::mem::swap(&mut self.get_column_mut(first_col).0[row], buffer);
        }
        self
    }
    fn mix_columns(&mut self) -> &mut Self {
        for word in self.0.iter_mut() {
            *word = word.mix_columns();
        }
        self
    }
    fn inv_mix_columns(&mut self) -> &mut Self {
        for word in self.0.iter_mut() {
            *word = word.inv_mix_columns();
        }
        self
    }
    fn add_round_key(&mut self, round_key: Block) -> &mut Self {
        for word in 0..4 {
            self.0[word] += round_key.0[word];
        }
        self
    }
    fn cipher(mut self, round_keys: &[Block; NUM_ROUNDS + 1]) -> Block {
        self.add_round_key(round_keys[0]);
        for round in 1..NUM_ROUNDS {
            self.sub_bytes()
                .shift_rows()
                .mix_columns()
                .add_round_key(round_keys[round]);
        }
        self.sub_bytes()
            .shift_rows()
            .add_round_key(round_keys[NUM_ROUNDS]);
        self
    }
    fn inv_cipher(mut self, round_keys: &[Block; NUM_ROUNDS + 1]) -> Block {
        self.add_round_key(round_keys[NUM_ROUNDS]);
        for round in (1..NUM_ROUNDS).rev() {
            self.inv_shift_rows()
                .inv_sub_bytes()
                .add_round_key(round_keys[round])
                .inv_mix_columns();
        }
        self.inv_shift_rows()
            .inv_sub_bytes()
            .add_round_key(round_keys[0]);
        self
    }
}
impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in 0..4 {
            writeln!(
                f,
                "{:02x} {:02x} {:02x} {:02x}",
                self.get_column(0).0[row].0,
                self.get_column(1).0[row].0,
                self.get_column(2).0[row].0,
                self.get_column(3).0[row].0
            )?;
        }
        Ok(())
    }
}
impl std::ops::Add for Block {
    type Output = Block;
    fn add(self, rhs: Self) -> Self::Output {
        Block([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
        ])
    }
}
impl std::ops::AddAssign for Block {
    fn add_assign(&mut self, rhs: Self) {
        self.0[0] += rhs.0[0];
        self.0[1] += rhs.0[1];
        self.0[2] += rhs.0[2];
        self.0[3] += rhs.0[3];
    }
}
impl std::ops::BitXor for Block {
    type Output = Block;
    fn bitxor(self, rhs: Self) -> Self::Output {
        let mut out = Block([Word([Byte(7); 4]); 4]);
        for col in 0..4 {
            for row in 0..4 {
                out.0[col].0[row] = Byte(self.0[col].0[row].0 ^ rhs.0[col].0[row].0)
            }
        }
        out
    }
}
impl std::ops::BitXorAssign for Block {
    fn bitxor_assign(&mut self, rhs: Self) {
        for col in 0..4 {
            for row in 0..4 {
                self.0[col].0[row].0 ^= rhs.0[col].0[row].0;
            }
        }
    }
}

/// Gets the key if given a valid input or None if it is too small
fn key_expansion(key: &[u8]) -> Option<[Block; NUM_ROUNDS + 1]> {
    if key.len() < 32 {
        return None;
    }
    let mut round_keys = (0..10).map(|shift| Word([Byte(1) << shift, Byte(0), Byte(0), Byte(0)]));
    let mut filled = 0;
    let mut expanded = [Word([Byte(0); 4]); 4 * (NUM_ROUNDS + 1)];
    let key_word_len = key.len().div_ceil(4);
    // The first Nk words of the output are to be the input key
    while filled < key_word_len && filled < expanded.len() {
        for offset in 0..4 {
            expanded[filled].0[offset] = Byte(key.get((filled * 4) + offset).copied().unwrap_or(0));
        }
        filled += 1;
    }

    while filled < expanded.len() {
        // If i%Nk = 0
        expanded[filled] = expanded[filled - key_word_len]
            + if filled.is_multiple_of(key_word_len) {
                expanded[filled - 1].rot_word().sub_word() + round_keys.next().unwrap()
            } else if (filled + 4).is_multiple_of(8) {
                expanded[filled - 1].sub_word()
            } else {
                expanded[filled - 1]
            };
        filled += 1
    }

    let mut out = [Block([Word([Byte(0); 4]); 4]); NUM_ROUNDS + 1];
    for index in 0..(NUM_ROUNDS + 1) {
        out[index].0[0] = expanded[index * 4];
        out[index].0[1] = expanded[(index * 4) + 1];
        out[index].0[2] = expanded[(index * 4) + 2];
        out[index].0[3] = expanded[(index * 4) + 3];
    }
    Some(out)
}

static S_BOX: [Byte; 256] = [
    Byte(0x63), // 0
    Byte(0x7c), // 1
    Byte(0x77), // 2
    Byte(0x7b), // 3
    Byte(0xf2), // 4
    Byte(0x6b), // 5
    Byte(0x6f), // 6
    Byte(0xc5), // 7
    Byte(0x30), // 8
    Byte(0x01), // 9
    Byte(0x67), // 10
    Byte(0x2b), // 11
    Byte(0xfe), // 12
    Byte(0xd7), // 13
    Byte(0xab), // 14
    Byte(0x76), // 15
    Byte(0xca), // 16
    Byte(0x82), // 17
    Byte(0xc9), // 18
    Byte(0x7d), // 19
    Byte(0xfa), // 20
    Byte(0x59), // 21
    Byte(0x47), // 22
    Byte(0xf0), // 23
    Byte(0xad), // 24
    Byte(0xd4), // 25
    Byte(0xa2), // 26
    Byte(0xaf), // 27
    Byte(0x9c), // 28
    Byte(0xa4), // 29
    Byte(0x72), // 30
    Byte(0xc0), // 31
    Byte(0xb7), // 32
    Byte(0xfd), // 33
    Byte(0x93), // 34
    Byte(0x26), // 35
    Byte(0x36), // 36
    Byte(0x3f), // 37
    Byte(0xf7), // 38
    Byte(0xcc), // 39
    Byte(0x34), // 40
    Byte(0xa5), // 41
    Byte(0xe5), // 42
    Byte(0xf1), // 43
    Byte(0x71), // 44
    Byte(0xd8), // 45
    Byte(0x31), // 46
    Byte(0x15), // 47
    Byte(0x04), // 48
    Byte(0xc7), // 49
    Byte(0x23), // 50
    Byte(0xc3), // 51
    Byte(0x18), // 52
    Byte(0x96), // 53
    Byte(0x05), // 54
    Byte(0x9a), // 55
    Byte(0x07), // 56
    Byte(0x12), // 57
    Byte(0x80), // 58
    Byte(0xe2), // 59
    Byte(0xeb), // 60
    Byte(0x27), // 61
    Byte(0xb2), // 62
    Byte(0x75), // 63
    Byte(0x09), // 64
    Byte(0x83), // 65
    Byte(0x2c), // 66
    Byte(0x1a), // 67
    Byte(0x1b), // 68
    Byte(0x6e), // 69
    Byte(0x5a), // 70
    Byte(0xa0), // 71
    Byte(0x52), // 72
    Byte(0x3b), // 73
    Byte(0xd6), // 74
    Byte(0xb3), // 75
    Byte(0x29), // 76
    Byte(0xe3), // 77
    Byte(0x2f), // 78
    Byte(0x84), // 79
    Byte(0x53), // 80
    Byte(0xd1), // 81
    Byte(0x00), // 82
    Byte(0xed), // 83
    Byte(0x20), // 84
    Byte(0xfc), // 85
    Byte(0xb1), // 86
    Byte(0x5b), // 87
    Byte(0x6a), // 88
    Byte(0xcb), // 89
    Byte(0xbe), // 90
    Byte(0x39), // 91
    Byte(0x4a), // 92
    Byte(0x4c), // 93
    Byte(0x58), // 94
    Byte(0xcf), // 95
    Byte(0xd0), // 96
    Byte(0xef), // 97
    Byte(0xaa), // 98
    Byte(0xfb), // 99
    Byte(0x43), // 100
    Byte(0x4d), // 101
    Byte(0x33), // 102
    Byte(0x85), // 103
    Byte(0x45), // 104
    Byte(0xf9), // 105
    Byte(0x02), // 106
    Byte(0x7f), // 107
    Byte(0x50), // 108
    Byte(0x3c), // 109
    Byte(0x9f), // 110
    Byte(0xa8), // 111
    Byte(0x51), // 112
    Byte(0xa3), // 113
    Byte(0x40), // 114
    Byte(0x8f), // 115
    Byte(0x92), // 116
    Byte(0x9d), // 117
    Byte(0x38), // 118
    Byte(0xf5), // 119
    Byte(0xbc), // 120
    Byte(0xb6), // 121
    Byte(0xda), // 122
    Byte(0x21), // 123
    Byte(0x10), // 124
    Byte(0xff), // 125
    Byte(0xf3), // 126
    Byte(0xd2), // 127
    Byte(0xcd), // 128
    Byte(0x0c), // 129
    Byte(0x13), // 130
    Byte(0xec), // 131
    Byte(0x5f), // 132
    Byte(0x97), // 133
    Byte(0x44), // 134
    Byte(0x17), // 135
    Byte(0xc4), // 136
    Byte(0xa7), // 137
    Byte(0x7e), // 138
    Byte(0x3d), // 139
    Byte(0x64), // 140
    Byte(0x5d), // 141
    Byte(0x19), // 142
    Byte(0x73), // 143
    Byte(0x60), // 144
    Byte(0x81), // 145
    Byte(0x4f), // 146
    Byte(0xdc), // 147
    Byte(0x22), // 148
    Byte(0x2a), // 149
    Byte(0x90), // 150
    Byte(0x88), // 151
    Byte(0x46), // 152
    Byte(0xee), // 153
    Byte(0xb8), // 154
    Byte(0x14), // 155
    Byte(0xde), // 156
    Byte(0x5e), // 157
    Byte(0x0b), // 158
    Byte(0xdb), // 159
    Byte(0xe0), // 160
    Byte(0x32), // 161
    Byte(0x3a), // 162
    Byte(0x0a), // 163
    Byte(0x49), // 164
    Byte(0x06), // 165
    Byte(0x24), // 166
    Byte(0x5c), // 167
    Byte(0xc2), // 168
    Byte(0xd3), // 169
    Byte(0xac), // 170
    Byte(0x62), // 171
    Byte(0x91), // 172
    Byte(0x95), // 173
    Byte(0xe4), // 174
    Byte(0x79), // 175
    Byte(0xe7), // 176
    Byte(0xc8), // 177
    Byte(0x37), // 178
    Byte(0x6d), // 179
    Byte(0x8d), // 180
    Byte(0xd5), // 181
    Byte(0x4e), // 182
    Byte(0xa9), // 183
    Byte(0x6c), // 184
    Byte(0x56), // 185
    Byte(0xf4), // 186
    Byte(0xea), // 187
    Byte(0x65), // 188
    Byte(0x7a), // 189
    Byte(0xae), // 190
    Byte(0x08), // 191
    Byte(0xba), // 192
    Byte(0x78), // 193
    Byte(0x25), // 194
    Byte(0x2e), // 195
    Byte(0x1c), // 196
    Byte(0xa6), // 197
    Byte(0xb4), // 198
    Byte(0xc6), // 199
    Byte(0xe8), // 200
    Byte(0xdd), // 201
    Byte(0x74), // 202
    Byte(0x1f), // 203
    Byte(0x4b), // 204
    Byte(0xbd), // 205
    Byte(0x8b), // 206
    Byte(0x8a), // 207
    Byte(0x70), // 208
    Byte(0x3e), // 209
    Byte(0xb5), // 210
    Byte(0x66), // 211
    Byte(0x48), // 212
    Byte(0x03), // 213
    Byte(0xf6), // 214
    Byte(0x0e), // 215
    Byte(0x61), // 216
    Byte(0x35), // 217
    Byte(0x57), // 218
    Byte(0xb9), // 219
    Byte(0x86), // 220
    Byte(0xc1), // 221
    Byte(0x1d), // 222
    Byte(0x9e), // 223
    Byte(0xe1), // 224
    Byte(0xf8), // 225
    Byte(0x98), // 226
    Byte(0x11), // 227
    Byte(0x69), // 228
    Byte(0xd9), // 229
    Byte(0x8e), // 230
    Byte(0x94), // 231
    Byte(0x9b), // 232
    Byte(0x1e), // 233
    Byte(0x87), // 234
    Byte(0xe9), // 235
    Byte(0xce), // 236
    Byte(0x55), // 237
    Byte(0x28), // 238
    Byte(0xdf), // 239
    Byte(0x8c), // 240
    Byte(0xa1), // 241
    Byte(0x89), // 242
    Byte(0x0d), // 243
    Byte(0xbf), // 244
    Byte(0xe6), // 245
    Byte(0x42), // 246
    Byte(0x68), // 247
    Byte(0x41), // 248
    Byte(0x99), // 249
    Byte(0x2d), // 250
    Byte(0x0f), // 251
    Byte(0xb0), // 252
    Byte(0x54), // 253
    Byte(0xbb), // 254
    Byte(0x16), // 255
];
static INV_S_BOX: [Byte; 256] = [
    Byte(0x52), // 0
    Byte(0x09), // 1
    Byte(0x6a), // 2
    Byte(0xd5), // 3
    Byte(0x30), // 4
    Byte(0x36), // 5
    Byte(0xa5), // 6
    Byte(0x38), // 7
    Byte(0xbf), // 8
    Byte(0x40), // 9
    Byte(0xa3), // 10
    Byte(0x9e), // 11
    Byte(0x81), // 12
    Byte(0xf3), // 13
    Byte(0xd7), // 14
    Byte(0xfb), // 15
    Byte(0x7c), // 16
    Byte(0xe3), // 17
    Byte(0x39), // 18
    Byte(0x82), // 19
    Byte(0x9b), // 20
    Byte(0x2f), // 21
    Byte(0xff), // 22
    Byte(0x87), // 23
    Byte(0x34), // 24
    Byte(0x8e), // 25
    Byte(0x43), // 26
    Byte(0x44), // 27
    Byte(0xc4), // 28
    Byte(0xde), // 29
    Byte(0xe9), // 30
    Byte(0xcb), // 31
    Byte(0x54), // 32
    Byte(0x7b), // 33
    Byte(0x94), // 34
    Byte(0x32), // 35
    Byte(0xa6), // 36
    Byte(0xc2), // 37
    Byte(0x23), // 38
    Byte(0x3d), // 39
    Byte(0xee), // 40
    Byte(0x4c), // 41
    Byte(0x95), // 42
    Byte(0x0b), // 43
    Byte(0x42), // 44
    Byte(0xfa), // 45
    Byte(0xc3), // 46
    Byte(0x4e), // 47
    Byte(0x08), // 48
    Byte(0x2e), // 49
    Byte(0xa1), // 50
    Byte(0x66), // 51
    Byte(0x28), // 52
    Byte(0xd9), // 53
    Byte(0x24), // 54
    Byte(0xb2), // 55
    Byte(0x76), // 56
    Byte(0x5b), // 57
    Byte(0xa2), // 58
    Byte(0x49), // 59
    Byte(0x6d), // 60
    Byte(0x8b), // 61
    Byte(0xd1), // 62
    Byte(0x25), // 63
    Byte(0x72), // 64
    Byte(0xf8), // 65
    Byte(0xf6), // 66
    Byte(0x64), // 67
    Byte(0x86), // 68
    Byte(0x68), // 69
    Byte(0x98), // 70
    Byte(0x16), // 71
    Byte(0xd4), // 72
    Byte(0xa4), // 73
    Byte(0x5c), // 74
    Byte(0xcc), // 75
    Byte(0x5d), // 76
    Byte(0x65), // 77
    Byte(0xb6), // 78
    Byte(0x92), // 79
    Byte(0x6c), // 80
    Byte(0x70), // 81
    Byte(0x48), // 82
    Byte(0x50), // 83
    Byte(0xfd), // 84
    Byte(0xed), // 85
    Byte(0xb9), // 86
    Byte(0xda), // 87
    Byte(0x5e), // 88
    Byte(0x15), // 89
    Byte(0x46), // 90
    Byte(0x57), // 91
    Byte(0xa7), // 92
    Byte(0x8d), // 93
    Byte(0x9d), // 94
    Byte(0x84), // 95
    Byte(0x90), // 96
    Byte(0xd8), // 97
    Byte(0xab), // 98
    Byte(0x00), // 99
    Byte(0x8c), // 100
    Byte(0xbc), // 101
    Byte(0xd3), // 102
    Byte(0x0a), // 103
    Byte(0xf7), // 104
    Byte(0xe4), // 105
    Byte(0x58), // 106
    Byte(0x05), // 107
    Byte(0xb8), // 108
    Byte(0xb3), // 109
    Byte(0x45), // 110
    Byte(0x06), // 111
    Byte(0xd0), // 112
    Byte(0x2c), // 113
    Byte(0x1e), // 114
    Byte(0x8f), // 115
    Byte(0xca), // 116
    Byte(0x3f), // 117
    Byte(0x0f), // 118
    Byte(0x02), // 119
    Byte(0xc1), // 120
    Byte(0xaf), // 121
    Byte(0xbd), // 122
    Byte(0x03), // 123
    Byte(0x01), // 124
    Byte(0x13), // 125
    Byte(0x8a), // 126
    Byte(0x6b), // 127
    Byte(0x3a), // 128
    Byte(0x91), // 129
    Byte(0x11), // 130
    Byte(0x41), // 131
    Byte(0x4f), // 132
    Byte(0x67), // 133
    Byte(0xdc), // 134
    Byte(0xea), // 135
    Byte(0x97), // 136
    Byte(0xf2), // 137
    Byte(0xcf), // 138
    Byte(0xce), // 139
    Byte(0xf0), // 140
    Byte(0xb4), // 141
    Byte(0xe6), // 142
    Byte(0x73), // 143
    Byte(0x96), // 144
    Byte(0xac), // 145
    Byte(0x74), // 146
    Byte(0x22), // 147
    Byte(0xe7), // 148
    Byte(0xad), // 149
    Byte(0x35), // 150
    Byte(0x85), // 151
    Byte(0xe2), // 152
    Byte(0xf9), // 153
    Byte(0x37), // 154
    Byte(0xe8), // 155
    Byte(0x1c), // 156
    Byte(0x75), // 157
    Byte(0xdf), // 158
    Byte(0x6e), // 159
    Byte(0x47), // 160
    Byte(0xf1), // 161
    Byte(0x1a), // 162
    Byte(0x71), // 163
    Byte(0x1d), // 164
    Byte(0x29), // 165
    Byte(0xc5), // 166
    Byte(0x89), // 167
    Byte(0x6f), // 168
    Byte(0xb7), // 169
    Byte(0x62), // 170
    Byte(0x0e), // 171
    Byte(0xaa), // 172
    Byte(0x18), // 173
    Byte(0xbe), // 174
    Byte(0x1b), // 175
    Byte(0xfc), // 176
    Byte(0x56), // 177
    Byte(0x3e), // 178
    Byte(0x4b), // 179
    Byte(0xc6), // 180
    Byte(0xd2), // 181
    Byte(0x79), // 182
    Byte(0x20), // 183
    Byte(0x9a), // 184
    Byte(0xdb), // 185
    Byte(0xc0), // 186
    Byte(0xfe), // 187
    Byte(0x78), // 188
    Byte(0xcd), // 189
    Byte(0x5a), // 190
    Byte(0xf4), // 191
    Byte(0x1f), // 192
    Byte(0xdd), // 193
    Byte(0xa8), // 194
    Byte(0x33), // 195
    Byte(0x88), // 196
    Byte(0x07), // 197
    Byte(0xc7), // 198
    Byte(0x31), // 199
    Byte(0xb1), // 200
    Byte(0x12), // 201
    Byte(0x10), // 202
    Byte(0x59), // 203
    Byte(0x27), // 204
    Byte(0x80), // 205
    Byte(0xec), // 206
    Byte(0x5f), // 207
    Byte(0x60), // 208
    Byte(0x51), // 209
    Byte(0x7f), // 210
    Byte(0xa9), // 211
    Byte(0x19), // 212
    Byte(0xb5), // 213
    Byte(0x4a), // 214
    Byte(0x0d), // 215
    Byte(0x2d), // 216
    Byte(0xe5), // 217
    Byte(0x7a), // 218
    Byte(0x9f), // 219
    Byte(0x93), // 220
    Byte(0xc9), // 221
    Byte(0x9c), // 222
    Byte(0xef), // 223
    Byte(0xa0), // 224
    Byte(0xe0), // 225
    Byte(0x3b), // 226
    Byte(0x4d), // 227
    Byte(0xae), // 228
    Byte(0x2a), // 229
    Byte(0xf5), // 230
    Byte(0xb0), // 231
    Byte(0xc8), // 232
    Byte(0xeb), // 233
    Byte(0xbb), // 234
    Byte(0x3c), // 235
    Byte(0x83), // 236
    Byte(0x53), // 237
    Byte(0x99), // 238
    Byte(0x61), // 239
    Byte(0x17), // 240
    Byte(0x2b), // 241
    Byte(0x04), // 242
    Byte(0x7e), // 243
    Byte(0xba), // 244
    Byte(0x77), // 245
    Byte(0xd6), // 246
    Byte(0x26), // 247
    Byte(0xe1), // 248
    Byte(0x69), // 249
    Byte(0x14), // 250
    Byte(0x63), // 251
    Byte(0x55), // 252
    Byte(0x21), // 253
    Byte(0x0c), // 254
    Byte(0x7d), // 255
];
