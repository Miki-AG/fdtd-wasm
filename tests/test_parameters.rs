use fdtd_wasm::parameters::{SimulationParameters, SourceDefinition, validate_parameters};

#[test]
fn test_validate_parameters_valid() {
    let params = SimulationParameters {
        width: 100,
        height: 100,
        source: SourceDefinition { x: 50, y: 50, amplitude: 1.0, frequency: 1.0 },
        obstacles: vec![],
        duration_steps: 100,
    };
    assert!(validate_parameters(&params).is_ok());
}

#[test]
fn test_validate_parameters_invalid_dimensions() {
    let params = SimulationParameters {
        width: 0,
        height: 100,
        source: SourceDefinition { x: 0, y: 0, amplitude: 1.0, frequency: 1.0 },
        obstacles: vec![],
        duration_steps: 100,
    };
    assert!(validate_parameters(&params).is_err());
}

#[test]
fn test_validate_parameters_source_out_of_bounds() {
    let params = SimulationParameters {
        width: 50,
        height: 50,
        source: SourceDefinition { x: 100, y: 100, amplitude: 1.0, frequency: 1.0 },
        obstacles: vec![],
        duration_steps: 100,
    };
    assert!(validate_parameters(&params).is_err());
}

#[test]
fn test_validate_parameters_source_on_boundary() {
    let params = SimulationParameters {
        width: 100,
        height: 100,
        source: SourceDefinition { x: 100, y: 50, amplitude: 1.0, frequency: 1.0 }, // x = width is invalid (0..99)
        obstacles: vec![],
        duration_steps: 100,
    };
    assert!(validate_parameters(&params).is_err());
}

#[test]
fn test_validate_parameters_negative_frequency() {
    let params = SimulationParameters {
        width: 100,
        height: 100,
        source: SourceDefinition { x: 50, y: 50, amplitude: 1.0, frequency: -5.0 },
        obstacles: vec![],
        duration_steps: 100,
    };
    assert!(validate_parameters(&params).is_err());
}

#[test]
fn test_validate_parameters_zero_duration() {
    let params = SimulationParameters {
        width: 100,
        height: 100,
        source: SourceDefinition { x: 50, y: 50, amplitude: 1.0, frequency: 1.0 },
        obstacles: vec![],
        duration_steps: 0,
    };
    assert!(validate_parameters(&params).is_err());
}