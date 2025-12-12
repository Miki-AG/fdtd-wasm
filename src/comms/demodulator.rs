pub struct Demodulator {
    samples_per_symbol: usize,
    sample_counter: usize,
    freq_0: f64,
    freq_1: f64,
    // I/Q Accumulators for Freq 0 and Freq 1
    i0: f64, q0: f64,
    i1: f64, q1: f64,
    received_bits: Vec<u8>,
    decoded_text: String,
}

impl Demodulator {
    pub fn new(freq_0: f64, freq_1: f64, samples_per_symbol: usize) -> Self {
        Self {
            samples_per_symbol,
            sample_counter: 0,
            freq_0,
            freq_1,
            i0: 0.0, q0: 0.0,
            i1: 0.0, q1: 0.0,
            received_bits: Vec::new(),
            decoded_text: String::new(),
        }
    }

    /// Processes a single field sample.
    /// Returns Some(bit) if a symbol period just finished and a bit was decided.
    pub fn process_sample(&mut self, value: f64, t: f64) -> Option<u8> {
        let omega0 = 2.0 * std::f64::consts::PI * self.freq_0;
        let omega1 = 2.0 * std::f64::consts::PI * self.freq_1;

        // Correlate
        self.i0 += value * (omega0 * t).cos();
        self.q0 += value * (omega0 * t).sin();
        self.i1 += value * (omega1 * t).cos();
        self.q1 += value * (omega1 * t).sin();

        self.sample_counter += 1;

        if self.sample_counter >= self.samples_per_symbol {
            // Decision time
            let energy0 = self.i0.powi(2) + self.q0.powi(2);
            let energy1 = self.i1.powi(2) + self.q1.powi(2);

            let bit = if energy1 > energy0 { 1 } else { 0 };

            // Store bit
            self.received_bits.push(bit);
            
            // Try decode text (every 8 bits)
            if self.received_bits.len() % 8 == 0 {
                let byte_idx = self.received_bits.len() / 8 - 1;
                let chunk = &self.received_bits[byte_idx * 8 .. (byte_idx + 1) * 8];
                let mut byte = 0u8;
                for (i, &b) in chunk.iter().enumerate() {
                    if b == 1 {
                        byte |= 1 << (7 - i); // MSB first
                    }
                }
                self.decoded_text.push(byte as char);
            }

            // Reset
            self.sample_counter = 0;
            self.i0 = 0.0; self.q0 = 0.0;
            self.i1 = 0.0; self.q1 = 0.0;

            return Some(bit);
        }

        None
    }

    pub fn get_text(&self) -> String {
        self.decoded_text.clone()
    }
    
    pub fn get_bits_string(&self) -> String {
        self.received_bits.iter().map(|b| b.to_string()).collect()
    }
}

pub fn bits_to_text(bits: &[u8]) -> String {
    let mut text = String::new();
    for chunk in bits.chunks(8) {
        if chunk.len() == 8 {
            let mut byte = 0u8;
            for (i, &b) in chunk.iter().enumerate() {
                if b == 1 {
                    byte |= 1 << (7 - i);
                }
            }
            text.push(byte as char);
        }
    }
    text
}
