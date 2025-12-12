use serde::{Deserialize, Serialize};

/// Represents the simulation configuration provided by the user.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimulationParameters {
    pub width: usize,
    pub height: usize,
    pub source: SourceDefinition,
    pub obstacles: Vec<String>, // SVG path strings
    pub duration_steps: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SourceDefinition {
    pub x: usize,
    pub y: usize,
    pub amplitude: f64,
    pub frequency: f64,
}

/// Validates the parameters (e.g., source within bounds).
pub fn validate_parameters(params: &SimulationParameters) -> Result<(), String> {
    todo!("Implement parameter validation")
}
