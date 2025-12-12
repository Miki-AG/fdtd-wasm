use fdtd_wasm::FdtdSimulator;
use fdtd_wasm::parameters::{SimulationParameters, SourceDefinition};
use wasm_bindgen::JsValue;
use serde_wasm_bindgen;

#[test]
fn test_fdtd_simulator_new_succeeds_with_valid_config() {
    let params = SimulationParameters {
        width: 10, height: 10,
        source: SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0 },
        obstacles: vec![],
        duration_steps: 100,
    };
    let config_json = serde_wasm_bindgen::to_value(&params).unwrap();
    
    let simulator = FdtdSimulator::new(config_json);
    assert!(simulator.is_ok());
    let simulator = simulator.unwrap();
    assert_eq!(simulator.get_current_step(), 0);
}

#[test]
fn test_fdtd_simulator_new_fails_with_invalid_config() {
    // Malformed JSON (e.g., missing width) or invalid parameters (e.g., width=0)
    let malformed_json = JsValue::from_str(r#"{"height": 10, "source": {"x":0,"y":0,"amplitude":1,"frequency":1}, "obstacles":[], "duration_steps":1}"#);
    let simulator = FdtdSimulator::new(malformed_json);
    assert!(simulator.is_err());

    let invalid_params = SimulationParameters {
        width: 0, height: 10,
        source: SourceDefinition { x: 0, y: 0, amplitude: 1.0, frequency: 1.0 },
        obstacles: vec![],
        duration_steps: 100,
    };
    let invalid_config_json = serde_wasm_bindgen::to_value(&invalid_params).unwrap();
    let simulator_invalid = FdtdSimulator::new(invalid_config_json);
    assert!(simulator_invalid.is_err());
}

#[test]
fn test_fdtd_simulator_get_current_step() {
    let params = SimulationParameters {
        width: 10, height: 10,
        source: SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0 },
        obstacles: vec![],
        duration_steps: 100,
    };
    let config_json = serde_wasm_bindgen::to_value(&params).unwrap();
    let simulator = FdtdSimulator::new(config_json).unwrap();
    
    assert_eq!(simulator.get_current_step(), 0);
}

#[test]
fn test_fdtd_simulator_step_advances_time() {
    let params = SimulationParameters {
        width: 10, height: 10,
        source: SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0 },
        obstacles: vec![],
        duration_steps: 100,
    };
    let config_json = serde_wasm_bindgen::to_value(&params).unwrap();
    let mut simulator = FdtdSimulator::new(config_json).unwrap();
    
    assert_eq!(simulator.get_current_step(), 0);
    simulator.step();
    assert_eq!(simulator.get_current_step(), 1);
}

#[test]
fn test_fdtd_simulator_get_frame_buffer_size() {
    let params = SimulationParameters {
        width: 10, height: 10,
        source: SourceDefinition { x: 5, y: 5, amplitude: 1.0, frequency: 1.0 },
        obstacles: vec![],
        duration_steps: 100,
    };
    let config_json = serde_wasm_bindgen::to_value(&params).unwrap();
    let simulator = FdtdSimulator::new(config_json).unwrap();
    
    let buffer = simulator.get_frame_buffer();
    assert_eq!(buffer.len(), 10 * 10 * 4); // RGBA
}
