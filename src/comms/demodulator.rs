use crate::comms::packet::{PacketDecoder, PacketState};

pub struct Demodulator {
    samples_per_symbol: usize,
    sample_counter: usize,
    freq_0: f64,
    freq_1: f64,
    i0: f64, q0: f64,
    i1: f64, q1: f64,
    
    decoder: PacketDecoder,
    last_message: String,
    
    // Debugging info
    received_bits_debug: String,
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
            decoder: PacketDecoder::new(),
            last_message: String::new(),
            received_bits_debug: String::new(),
        }
    }

    pub fn process_sample(&mut self, value: f64, t: f64) -> Option<u8> {
        let omega0 = 2.0 * std::f64::consts::PI * self.freq_0;
        let omega1 = 2.0 * std::f64::consts::PI * self.freq_1;

        self.i0 += value * (omega0 * t).cos();
        self.q0 += value * (omega0 * t).sin();
        self.i1 += value * (omega1 * t).cos();
        self.q1 += value * (omega1 * t).sin();

        self.sample_counter += 1;

        if self.sample_counter >= self.samples_per_symbol {
            let energy0 = self.i0.powi(2) + self.q0.powi(2);
            let energy1 = self.i1.powi(2) + self.q1.powi(2);

            // Squelch
            if energy0 + energy1 < 1.0 {
                self.reset_accumulators();
                return None;
            }

            let bit = if energy1 > energy0 { 1 } else { 0 };
            
            // Debug
            self.received_bits_debug.push(if bit == 1 { '1' } else { '0' });
            if self.received_bits_debug.len() > 64 {
                self.received_bits_debug.drain(0..1); // Keep last 64
            }

            // Feed Packet Decoder
            if let Some(msg) = self.decoder.push_bit(bit) {
                self.last_message = msg;
            }

            self.reset_accumulators();
            return Some(bit);
        }

        None
    }
    
    fn reset_accumulators(&mut self) {
        self.sample_counter = 0;
        self.i0 = 0.0; self.q0 = 0.0;
        self.i1 = 0.0; self.q1 = 0.0;
    }

    pub fn get_text(&self) -> String {
        self.last_message.clone()
    }
    
    pub fn get_bits_string(&self) -> String {
        self.received_bits_debug.clone()
    }
    
    pub fn get_state_string(&self) -> String {
        format!("{:?}", self.decoder.get_state())
    }
}

pub fn bits_to_text(bits: &[u8]) -> String {
    // Legacy helper, might remove
    String::new() 
}
