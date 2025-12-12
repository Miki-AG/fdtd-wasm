use fdtd_wasm::comms::modulator::{Modulator, text_to_bits, ModulationScheme};
use fdtd_wasm::comms::demodulator::Demodulator;

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
fn test_modulator_sequence_fsk() {
    let mut modulator = Modulator::new(1.0, 2.0, 10);
    modulator.load_text("A"); 
    
    // First byte 0xAA (10101010).
    // Bit 0: 1 -> FSK: (Freq 2.0, Amp 1.0)
    for _ in 0..10 {
        assert_eq!(modulator.next_modulation(), Some((2.0, 1.0)));
    }
    
    // Bit 1: 0 -> FSK: (Freq 1.0, Amp 1.0)
    for _ in 0..10 {
        assert_eq!(modulator.next_modulation(), Some((1.0, 1.0)));
    }
}

#[test]
fn test_modulator_ask() {
    let mut modulator = Modulator::new(1.0, 2.0, 10);
    modulator.set_scheme(ModulationScheme::ASK);
    modulator.load_text("A");
    
    // First byte 0xAA.
    // Bit 0: 1 -> ASK: (Freq 2.0 (F1), Amp 1.0)
    for _ in 0..10 {
        assert_eq!(modulator.next_modulation(), Some((2.0, 1.0)));
    }
    
    // Bit 1: 0 -> ASK: (Freq 2.0 (F1), Amp 0.0)
    for _ in 0..10 {
        assert_eq!(modulator.next_modulation(), Some((2.0, 0.0)));
    }
}

#[test]
fn test_demodulator_perfect_signal_fsk() {
    let f0 = 0.1;
    let f1 = 0.2;
    let samples_per_sym = 100;
    let mut dem = Demodulator::new(f0, f1, samples_per_sym);
    dem.set_scheme(ModulationScheme::FSK);
    
    // Simulate bit '0' (F0)
    for t_step in 0..samples_per_sym {
        let val = (2.0 * std::f64::consts::PI * f0 * (t_step as f64)).sin();
        let bit = dem.process_sample(val, t_step as f64);
        if t_step == samples_per_sym - 1 {
            assert_eq!(bit, Some(0));
        } else {
            assert_eq!(bit, None);
        }
    }
    
    // Simulate bit '1' (F1)
    for t_step in 0..samples_per_sym {
        let val = (2.0 * std::f64::consts::PI * f1 * (t_step as f64 + samples_per_sym as f64)).sin();
        let bit = dem.process_sample(val, (t_step as f64 + samples_per_sym as f64));
        if t_step == samples_per_sym - 1 {
            assert_eq!(bit, Some(1));
        } else {
            assert_eq!(bit, None);
        }
    }
}

#[test]
fn test_demodulator_perfect_signal_ask() {
    let f_carrier = 0.15; // ASK Carrier
    let samples_per_sym = 100;
    let mut dem = Demodulator::new(0.0, f_carrier, samples_per_sym); // f0 not used for ASK, f1 is carrier
    dem.set_scheme(ModulationScheme::ASK);

    // Simulate bit '1' (Carrier ON)
    for t_step in 0..samples_per_sym {
        let val = (2.0 * std::f64::consts::PI * f_carrier * (t_step as f64)).sin();
        let bit = dem.process_sample(val, t_step as f64);
        if t_step == samples_per_sym - 1 {
            assert_eq!(bit, Some(1));
        } else {
            assert_eq!(bit, None);
        }
    }

    // Simulate bit '0' (Carrier OFF / Silence)
    for t_step in 0..samples_per_sym {
        let val = 0.0; // No signal
        let bit = dem.process_sample(val, (t_step as f64 + samples_per_sym as f64));
        if t_step == samples_per_sym - 1 {
            assert_eq!(bit, Some(0));
        } else {
            assert_eq!(bit, None);
        }
    }
}
