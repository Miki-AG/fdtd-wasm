use fdtd_wasm::engine::compute_source_signal;
use fdtd_wasm::parameters::SignalType;
use std::f64::consts::PI;

#[test]
fn test_compute_source_signal_zero_time() {
    let val = compute_source_signal(0.0, 1.0, 10.0, &SignalType::ContinuousSine);
    assert_eq!(val, 0.0);
}

#[test]
fn test_compute_source_signal_peak() {
    // sin(2 * pi * f * t) -> max at t = 1/(4f)
    let f = 2.0;
    let t = 1.0 / (4.0 * f);
    let val = compute_source_signal(t, f, 1.0, &SignalType::ContinuousSine);
    assert!((val - 1.0).abs() < 1e-6);
}

#[test]
fn test_compute_source_signal_valley() {
    // sin(2 * pi * f * t) -> min at t = 3/(4f)
    let f = 2.0;
    let t = 3.0 / (4.0 * f);
    let val = compute_source_signal(t, f, 1.0, &SignalType::ContinuousSine);
    assert!((val - -1.0).abs() < 1e-6);
}

#[test]
fn test_compute_source_signal_zero_crossing() {
    // sin(2 * pi * f * t) -> 0 at t = 1/(2f)
    let f = 2.0;
    let t = 1.0 / (2.0 * f);
    let val = compute_source_signal(t, f, 1.0, &SignalType::ContinuousSine);
    assert!(val.abs() < 1e-6);
}

#[test]
fn test_compute_source_signal_square() {
    let f = 1.0;
    let t_high = 0.2; // First half
    let t_low = 0.7;  // Second half
    
    let val_high = compute_source_signal(t_high, f, 1.0, &SignalType::ContinuousSquare);
    assert_eq!(val_high, 1.0);
    
    let val_low = compute_source_signal(t_low, f, 1.0, &SignalType::ContinuousSquare);
    assert_eq!(val_low, -1.0);
}

#[test]
fn test_compute_source_signal_pulse_sine_cutoff() {
    let f = 1.0;
    let t_inside = 0.5;
    let t_outside = 1.5; // > 1/f
    
    let val_in = compute_source_signal(t_inside, f, 1.0, &SignalType::PulseSine);
    assert!(val_in.abs() > 0.0); // Should be active
    
    let val_out = compute_source_signal(t_outside, f, 1.0, &SignalType::PulseSine);
    assert_eq!(val_out, 0.0);
}
