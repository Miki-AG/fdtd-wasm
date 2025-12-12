use fdtd_wasm::state::SimulationState;

#[test]
fn test_state_new_initialization() {
    let width = 10;
    let height = 20;
    let state = SimulationState::new(width, height);

    assert_eq!(state.width, width);
    assert_eq!(state.height, height);
    assert_eq!(state.ez.len(), width * height);
    assert_eq!(state.hx.len(), width * height);
    assert_eq!(state.hy.len(), width * height);
    assert_eq!(state.materials.len(), width * height);
    assert_eq!(state.time_step, 0);

    // Check zero initialization
    assert!(state.ez.iter().all(|&x| x == 0.0));
}

#[test]
fn test_state_new_zero_size() {
    // Should be valid to create, just empty vectors
    let state = SimulationState::new(0, 0);
    assert_eq!(state.width, 0);
    assert!(state.ez.is_empty());
}

#[test]
fn test_state_reset() {
    let width = 10;
    let height = 10;
    let mut state = SimulationState::new(width, height);
    
    // Manually dirty the state
    state.ez[0] = 1.0;
    state.hx[5] = -0.5;
    state.time_step = 100;
    
    state.reset();
    
    assert_eq!(state.time_step, 0);
    assert_eq!(state.ez[0], 0.0);
    assert_eq!(state.hx[5], 0.0);
    // Ensure all fields are zeroed
    assert!(state.ez.iter().all(|&x| x == 0.0));
}