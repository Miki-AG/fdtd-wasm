#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PacketState {
    SearchPreamble, // Waiting for 0xAA pattern
    SearchSync,     // Waiting for 0x7E
    ReadLength,     // Reading 1 byte length
    ReadPayload,    // Reading N bytes
    ReadCRC,        // Reading 1 byte CRC
    Done,           // Packet complete
}

pub struct PacketDecoder {
    state: PacketState,
    buffer: u8,         // Shift register for detecting patterns
    bit_count: usize,   // Bits read in current state
    
    length: u8,
    payload: Vec<u8>,
    crc: u8,
}

impl PacketDecoder {
    pub fn new() -> Self {
        Self {
            state: PacketState::SearchPreamble,
            buffer: 0,
            bit_count: 0,
            length: 0,
            payload: Vec::new(),
            crc: 0,
        }
    }

    pub fn push_bit(&mut self, bit: u8) -> Option<String> {
        // Shift bit into buffer (LSB in)
        self.buffer = (self.buffer << 1) | (bit & 1);
        
        match self.state {
            PacketState::SearchPreamble => {
                if self.buffer == 0xAA {
                    self.state = PacketState::SearchSync;
                }
            },
            PacketState::SearchSync => {
                if self.buffer == 0x7E {
                    self.state = PacketState::ReadLength;
                    self.bit_count = 0;
                }
            },
            PacketState::ReadLength => {
                self.bit_count += 1;
                if self.bit_count == 8 {
                    self.length = self.buffer;
                    self.state = PacketState::ReadPayload;
                    self.payload.clear();
                    self.bit_count = 0;
                    if self.length == 0 {
                        self.state = PacketState::ReadCRC; // Empty payload
                    }
                }
            },
            PacketState::ReadPayload => {
                self.bit_count += 1;
                if self.bit_count == 8 {
                    self.payload.push(self.buffer);
                    self.bit_count = 0;
                    if self.payload.len() == self.length as usize {
                        self.state = PacketState::ReadCRC;
                    }
                }
            },
            PacketState::ReadCRC => {
                self.bit_count += 1;
                if self.bit_count == 8 {
                    self.crc = self.buffer;
                    // Validate CRC
                    let sum: u16 = self.payload.iter().map(|&b| b as u16).sum();
                    let calc_crc = (sum % 256) as u8;
                    
                    self.state = PacketState::SearchPreamble; // Reset for next
                    
                    if self.crc == calc_crc {
                        return Some(String::from_utf8_lossy(&self.payload).to_string());
                    } else {
                        // CRC Error
                        return Some(format!("[CRC ERROR] Expected {:02X}, Got {:02X}", calc_crc, self.crc));
                    }
                }
            },
            PacketState::Done => {},
        }
        None
    }
    
    pub fn get_state(&self) -> PacketState {
        self.state
    }

    pub fn get_partial_payload(&self) -> String {
        String::from_utf8_lossy(&self.payload).to_string()
    }
}
