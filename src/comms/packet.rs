use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum PacketState {
    SearchPreamble, // Waiting for 0xAA pattern
    SearchSync,     // Waiting for 0x7E
    ReadLength,     // Reading 1 byte length
    ReadPayload,    // Reading N bytes
    ReadCRC,        // Reading 1 byte CRC
    Done,           // Packet complete
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DecodeEvent {
    pub label: String,
    pub bits: String,
    pub is_complete: bool,
    pub is_error: bool,
}

pub struct PacketDecoder {
    state: PacketState,
    buffer: u8,         // Shift register for detecting patterns
    bit_count: usize,   // Bits read in current state
    
    length: u8,
    payload: Vec<u8>,
    crc: u8,
    
    pub history: Vec<DecodeEvent>,
    pub current_bits_buffer: String,
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
            history: Vec::new(),
            current_bits_buffer: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.state = PacketState::SearchPreamble;
        self.bit_count = 0;
        self.buffer = 0;
        self.payload.clear();
        self.history.clear();
        self.current_bits_buffer.clear();
    }

    pub fn push_bit(&mut self, bit: u8) -> Option<String> {
        // Shift bit into buffer (LSB in)
        self.buffer = (self.buffer << 1) | (bit & 1);
        self.current_bits_buffer.push(if bit == 1 { '1' } else { '0' });
        
        match self.state {
            PacketState::SearchPreamble => {
                if self.buffer == 0xAA {
                    self.history.push(DecodeEvent {
                        label: "PRE".to_string(),
                        bits: self.current_bits_buffer.clone(),
                        is_complete: true,
                        is_error: false,
                    });
                    self.current_bits_buffer.clear();
                    self.state = PacketState::SearchSync;
                }
            },
            PacketState::SearchSync => {
                if self.buffer == 0x7E {
                    self.history.push(DecodeEvent {
                        label: "SYNC".to_string(),
                        bits: self.current_bits_buffer.clone(),
                        is_complete: true,
                        is_error: false,
                    });
                    self.current_bits_buffer.clear();
                    self.state = PacketState::ReadLength;
                    self.bit_count = 0;
                }
            },
            PacketState::ReadLength => {
                self.bit_count += 1;
                if self.bit_count == 8 {
                    self.length = self.buffer;
                    self.history.push(DecodeEvent {
                        label: "LEN".to_string(),
                        bits: self.current_bits_buffer.clone(),
                        is_complete: true,
                        is_error: false,
                    });
                    self.current_bits_buffer.clear();
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
                    let label = String::from_utf8_lossy(&[self.buffer]).to_string();
                    self.history.push(DecodeEvent {
                        label,
                        bits: self.current_bits_buffer.clone(),
                        is_complete: true,
                        is_error: false,
                    });
                    self.current_bits_buffer.clear();
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
                    let is_valid = self.crc == calc_crc;

                    self.history.push(DecodeEvent {
                        label: "CRC".to_string(),
                        bits: self.current_bits_buffer.clone(),
                        is_complete: true,
                        is_error: !is_valid,
                    });
                    self.current_bits_buffer.clear();
                    
                    self.state = PacketState::SearchPreamble; // Reset for next
                    
                    if is_valid {
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
