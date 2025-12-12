use fdtd_wasm::engine::compute_source_signal;
use std::f64::consts::PI;

#[test]
fn test_compute_source_signal_zero_time() {
    let val = compute_source_signal(0.0, 1.0, 10.0);
    assert_eq!(val, 0.0);
}

#[test]
fn test_compute_source_signal_peak() {
    // sin(2 * pi * f * t) -> max at t = 1/(4f)
    let f = 2.0;
    let t = 1.0 / (4.0 * f);
    let val = compute_source_signal(t, f, 1.0);
    assert!((val - 1.0).abs() < 1e-6);
}

#[test]
fn test_compute_source_signal_valley() {
    // sin(2 * pi * f * t) -> min at t = 3/(4f)
    let f = 2.0;
    let t = 3.0 / (4.0 * f);
    let val = compute_source_signal(t, f, 1.0);
    assert!((val - -1.0).abs() < 1e-6);
}

#[test]
fn test_compute_source_signal_zero_crossing() {
    // sin(2 * pi * f * t) -> 0 at t = 1/(2f)
    let f = 2.0;
    let t = 1.0 / (2.0 * f);
    let val = compute_source_signal(t, f, 1.0);
    assert!(val.abs() < 1e-6);
}