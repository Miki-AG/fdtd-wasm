use fdtd_wasm::FdtdSimulator;
use fdtd_wasm::parameters::{SimulationParameters, SourceDefinition, SignalType};
use wasm_bindgen::JsValue;
use serde_wasm_bindgen;
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn test_fdtd_simulator_new_succeeds_with_valid_config() {
    let params = SimulationParameters {
        width: 10, height: 10,
        source: SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0, signal_type: SignalType::ContinuousSine },
        obstacles: vec![],
        duration_steps: 100,
    };
    let config_json = serde_wasm_bindgen::to_value(&params).unwrap();
    
    let simulator = FdtdSimulator::new(config_json);
    assert!(simulator.is_ok());
    let simulator = simulator.unwrap();
    assert_eq!(simulator.get_current_step(), 0);
}

#[wasm_bindgen_test]
fn test_fdtd_simulator_new_fails_with_invalid_config() {
    let malformed_json = JsValue::from_str(r#"{"height": 10, "source": {"x":0,"y":0,"amplitude":1,"frequency":1}, "obstacles":[], "duration_steps":1}"#);
    let simulator = FdtdSimulator::new(malformed_json);
    assert!(simulator.is_err());

    let invalid_params = SimulationParameters {
        width: 0, height: 10,
        source: SourceDefinition { x: 0, y: 0, amplitude: 1.0, frequency: 1.0, signal_type: SignalType::ContinuousSine },
        obstacles: vec![],
        duration_steps: 100,
    };
    let invalid_config_json = serde_wasm_bindgen::to_value(&invalid_params).unwrap();
    let simulator_invalid = FdtdSimulator::new(invalid_config_json);
    assert!(simulator_invalid.is_err());
}

#[wasm_bindgen_test]
fn test_fdtd_simulator_get_current_step() {
    let params = SimulationParameters {
        width: 10, height: 10,
        source: SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0, signal_type: SignalType::ContinuousSine },
        obstacles: vec![],
        duration_steps: 100,
    };
    let config_json = serde_wasm_bindgen::to_value(&params).unwrap();
    let simulator = FdtdSimulator::new(config_json).unwrap();
    
    assert_eq!(simulator.get_current_step(), 0);
}

#[wasm_bindgen_test]
fn test_fdtd_simulator_step_advances_time() {
    let params = SimulationParameters {
        width: 10, height: 10,
        source: SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0, signal_type: SignalType::ContinuousSine },
        obstacles: vec![],
        duration_steps: 100,
    };
    let config_json = serde_wasm_bindgen::to_value(&params).unwrap();
    let mut simulator = FdtdSimulator::new(config_json).unwrap();
    
    assert_eq!(simulator.get_current_step(), 0);
    simulator.step();
    assert_eq!(simulator.get_current_step(), 1);
}

#[wasm_bindgen_test]
fn test_fdtd_simulator_comms() {
    let params = SimulationParameters {
        width: 10, height: 10,
        source: SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0, signal_type: SignalType::ContinuousSine },
        obstacles: vec![],
        duration_steps: 100,
    };
    let config_json = serde_wasm_bindgen::to_value(&params).unwrap();
    let mut simulator = FdtdSimulator::new(config_json).unwrap();
    
    simulator.send_message("A");
    
    // Check if bits are queued
    let bits = simulator.get_transmission_bits();
    // Packet starts with 10101010...
    assert!(bits.starts_with("10101010"));
    
    // Check initial status
    assert!(simulator.get_demodulator_status().contains("SearchPreamble"));
}