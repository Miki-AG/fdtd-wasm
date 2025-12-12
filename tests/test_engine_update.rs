use fdtd_wasm::engine::{update_hx, update_hy, update_e_fields, apply_source, apply_forced_source};
use fdtd_wasm::state::SimulationState;
use fdtd_wasm::parameters::{SourceDefinition, SignalType};

#[test]
fn test_update_fields_no_source_remains_zero() {
    let mut state = SimulationState::new(10, 10);
    update_hx(&mut state);
    update_hy(&mut state);
    update_e_fields(&mut state);
    
    // Everything should still be zero
    assert!(state.ez.iter().all(|&x| x == 0.0));
    assert!(state.hx.iter().all(|&x| x == 0.0));
    assert!(state.hy.iter().all(|&x| x == 0.0));
}

#[test]
fn test_apply_source_injects_value() {
    let mut state = SimulationState::new(10, 10);
    let source = SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0, signal_type: SignalType::ContinuousSine };
    
    // We need to advance time to t=0.25 (peak) to see injection, because sin(0)=0.
    // apply_source uses state.time_step.
    // If freq=1.0, period=1.0. Peak at t=0.25.
    // Since time_step is integer, we assume dt=1? Wait, in engine.rs:
    // let t = state.time_step as f64;
    // So 1 step = 1.0 unit time.
    // If frequency is 1.0, then omega = 2*pi.
    // t=0 -> sin(0) = 0.
    // t=1 -> sin(2pi) = 0.
    // We will never see a value if freq is integer and dt=1.0!
    // UNLESS we use small frequency like 0.1.
    // Or we manually set time_step to non-integer? No, it's usize.
    
    // Let's use frequency 0.25 -> period = 4. 
    // t=1 -> sin(2pi * 0.25 * 1) = sin(pi/2) = 1.
    
    let source_visible = SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 0.25, signal_type: SignalType::ContinuousSine };
    state.time_step = 1;
    
    apply_source(&mut state, &source_visible);
    
    // Check index 5,5 (5 * 10 + 5 = 55)
    let idx = 55;
    assert!((state.ez[idx] - 1.0).abs() < 1e-6);
}

#[test]
fn test_update_hx_logic() {
    let mut state = SimulationState::new(10, 10);
    // Setup state such that Hx should change (requires Ez gradients)
    state.ez[55] = 1.0; 
    // Hx[idx] -= 0.5 * (Ez[idx_up] - Ez[idx])
    // at y=4 (row above 55 is 65).
    // Hx at 55 (y=5, x=5) uses Ez(5,5) and Ez(6,5).
    // Ez(5,5)=1. Ez(6,5)=0.
    // Hx -= 0.5 * (0 - 1) = +0.5.
    
    update_hx(&mut state);
    assert_eq!(state.hx[55], 0.5);
}

#[test]
fn test_update_hy_logic() {
    let mut state = SimulationState::new(10, 10);
    state.ez[55] = 1.0;
    // Hy[idx] += 0.5 * (Ez[right] - Ez[idx])
    // Hy at 55 (5,5) uses Ez(5,6) and Ez(5,5).
    // Ez(5,6)=0. Ez(5,5)=1.
    // Hy += 0.5 * (0 - 1) = -0.5.
    update_hy(&mut state);
    assert_eq!(state.hy[55], -0.5);
}

#[test]
fn test_update_e_fields_logic() {
    let mut state = SimulationState::new(10, 10);
    state.hx[55] = 1.0;
    // Ez update depends on Hx gradients.
    // Ez[55] uses ... - (Hx[55] - Hx[45])
    // Hx[55]=1. Hx[45]=0. Diff = 1.
    // Ez += 0.5 * (0 - 1) = -0.5.
    update_e_fields(&mut state);
    assert_eq!(state.ez[55], -0.5);
}

#[test]
fn test_apply_forced_source() {
    let mut state = SimulationState::new(10, 10);
    // Source at (5,5)
    apply_forced_source(&mut state, 5, 5, 10.0);
    
    // Check if value was added
    assert_eq!(state.ez[55], 10.0);
    
    // Apply again to verify accumulation (it's +=)
    apply_forced_source(&mut state, 5, 5, 5.0);
    assert_eq!(state.ez[55], 15.0);
}