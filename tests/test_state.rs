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
