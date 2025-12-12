/// Converts a string to a vector of bits (0s and 1s).
/// E.g., 'A' (0x41 = 01000001) -> [0, 1, 0, 0, 0, 0, 0, 1]
pub fn text_to_bits(text: &str) -> Vec<u8> {
    let mut bits = Vec::new();
    for byte in text.bytes() {
        for i in (0..8).rev() {
            bits.push((byte >> i) & 1);
        }
    }
    bits
}

pub struct Modulator {
    bits: Vec<u8>,
    current_bit_idx: usize,
    samples_per_symbol: usize,
    sample_counter: usize,
    freq_0: f64,
    freq_1: f64,
}

impl Modulator {
    pub fn new(freq_0: f64, freq_1: f64, samples_per_symbol: usize) -> Self {
        Self {
            bits: Vec::new(),
            current_bit_idx: 0,
            samples_per_symbol,
            sample_counter: 0,
            freq_0,
            freq_1,
        }
    }

    pub fn load_text(&mut self, text: &str) {
        // Construct Packet
        let mut data = Vec::new();
        data.push(0xAA); // Preamble
        data.push(0x7E); // Sync
        let len = text.len().min(255) as u8;
        data.push(len);  // Length
        
        let payload_bytes = text.as_bytes();
        data.extend_from_slice(&payload_bytes[0..len as usize]);
        
        // CRC (Simple Sum)
        let sum: u16 = payload_bytes.iter().map(|&b| b as u16).sum();
        data.push((sum % 256) as u8);
        
        // Convert to bits (MSB first logic needed for packet decoder?)
        // Decoder shifts << 1 | bit. So MSB first.
        // My previous text_to_bits was MSB first (7..0). Good.
        
        self.bits.clear();
        for byte in data {
            for i in (0..8).rev() {
                self.bits.push((byte >> i) & 1);
            }
        }
        
        self.current_bit_idx = 0;
        self.sample_counter = 0;
    }

    /// Returns the frequency for the current time step.
    /// Returns None if transmission is idle.
    pub fn next_frequency(&mut self) -> Option<f64> {
        if self.current_bit_idx >= self.bits.len() {
            return None;
        }

        let bit = self.bits[self.current_bit_idx];
        let freq = if bit == 1 { self.freq_1 } else { self.freq_0 };

        self.sample_counter += 1;
        if self.sample_counter >= self.samples_per_symbol {
            self.sample_counter = 0;
            self.current_bit_idx += 1;
        }

        Some(freq)
    }

    pub fn get_bits_string(&self) -> String {
        self.bits.iter().map(|b| b.to_string()).collect()
    }
}
