use fdtd_wasm::step::step;
use fdtd_wasm::parameters::{SimulationParameters, SourceDefinition, SignalType};
use fdtd_wasm::state::SimulationState;

#[test]
fn test_step_advances_time() {
    let params = SimulationParameters {
        width: 10, height: 10,
        source: SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0, signal_type: SignalType::ContinuousSine },
        obstacles: vec![],
        duration_steps: 100,
    };
    let mut state = SimulationState::new(10, 10);
    
    assert_eq!(state.time_step, 0);
    step(&params, &mut state);
    assert_eq!(state.time_step, 1);
    step(&params, &mut state);
    assert_eq!(state.time_step, 2);
}