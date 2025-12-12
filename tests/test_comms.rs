use fdtd_wasm::comms::modulator::{Modulator, text_to_bits};
use fdtd_wasm::comms::demodulator::{Demodulator, bits_to_text};

#[test]
fn test_text_to_bits_conversion() {
    let text = "AB"; 
    // 'A' = 65 = 01000001
    // 'B' = 66 = 01000010
    let bits = text_to_bits(text);
    assert_eq!(bits.len(), 16);
    assert_eq!(bits[0..8], [0, 1, 0, 0, 0, 0, 0, 1]);
    assert_eq!(bits[8..16], [0, 1, 0, 0, 0, 0, 1, 0]);
}

#[test]
fn test_bits_to_text_conversion() {
    let bits = vec![0, 1, 0, 0, 0, 0, 0, 1]; // 'A'
    let text = bits_to_text(&bits);
    assert_eq!(text, "A");
}

#[test]
fn test_modulator_sequence() {
    // 10 samples per bit
    let mut modulator = Modulator::new(1.0, 2.0, 10);
    modulator.load_text("A"); // 01...
    
    // First bit is 0 -> freq 1.0
    for _ in 0..10 {
        assert_eq!(modulator.next_frequency(), Some(1.0));
    }
    
    // Second bit is 1 -> freq 2.0
    for _ in 0..10 {
        assert_eq!(modulator.next_frequency(), Some(2.0));
    }
}

#[test]
fn test_demodulator_perfect_signal() {
    // Simple verification: Feed explicit perfect sine waves and check bit detection
    // F0 = 0.1, F1 = 0.2. Samples/Symbol = 100.
    // 1.0 / 0.1 = 10 samples/cycle. 10 cycles/symbol.
    // 1.0 / 0.2 = 5 samples/cycle. 20 cycles/symbol.
    
    let f0 = 0.1;
    let f1 = 0.2;
    let samples = 100;
    let mut dem = Demodulator::new(f0, f1, samples);
    
    // Simulate bit '0' (F0)
    for t in 0..samples {
        let val = (2.0 * std::f64::consts::PI * f0 * t as f64).sin();
        let bit = dem.process_sample(val, t as f64);
        if t == samples - 1 {
            assert_eq!(bit, Some(0));
        } else {
            assert_eq!(bit, None);
        }
    }
    
    // Simulate bit '1' (F1)
    for t in 0..samples {
        let val = (2.0 * std::f64::consts::PI * f1 * t as f64).sin();
        let bit = dem.process_sample(val, (t + samples) as f64); // Continue time
        if t == samples - 1 {
            assert_eq!(bit, Some(1));
        } else {
            assert_eq!(bit, None);
        }
    }
}
