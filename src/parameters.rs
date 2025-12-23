use serde::{Deserialize, Serialize};

/// Represents the simulation configuration provided by the user.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimulationParameters {
    pub width: usize,
    pub height: usize,
    pub source: SourceDefinition,
    pub comms: CommsDefinition,
    pub obstacles: Vec<String>, // SVG path strings
    pub duration_steps: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SignalType {
    ContinuousSine,
    ContinuousSquare,
    PulseSine,
    PulseSquare,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceDefinition {
    pub x: usize,
    pub y: usize,
    pub amplitude: f64,
    pub frequency: f64,
    pub signal_type: SignalType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommsDefinition {
    pub carrier_frequency: f64,
    pub deviation: f64,
    pub symbol_duration: usize,
}

/// Validates the parameters (e.g., source within bounds).
pub fn validate_parameters(params: &SimulationParameters) -> Result<(), String> {
    if params.width == 0 || params.height == 0 {
        return Err("Width and height must be greater than 0".to_string());
    }
    // Source position must be 0-indexed and strictly less than width/height
    if params.source.x >= params.width || params.source.y >= params.height {
        return Err(format!("Source position ({}, {}) must be within simulation bounds (0..{}x0..{})",
                           params.source.x, params.source.y, params.width - 1, params.height - 1));
    }
    if params.source.frequency <= 0.0 {
        return Err("Source frequency must be greater than 0".to_string());
    }
    if params.comms.symbol_duration == 0 {
        return Err("Symbol duration must be greater than 0".to_string());
    }
    if params.duration_steps == 0 {
        return Err("Duration steps must be greater than 0".to_string());
    }
    Ok(())
}
