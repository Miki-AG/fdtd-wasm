use crate::parameters::SimulationParameters;

/// Represents the state of the simulation at a specific time step.
/// Typically holds Ez (Electric field) and Hx, Hy (Magnetic fields).
/// For 2D TMz mode.
pub struct SimulationState {
    pub width: usize,
    pub height: usize,
    
    // Fields
    pub ez: Vec<f64>,
    pub hx: Vec<f64>,
    pub hy: Vec<f64>,

    // Material properties (0.0 for free space, 1.0 for PEC, or intermediate for other materials)
    // For this project: Boolean mask or coefficient.
    // Let's use a coefficient array for flexibility (e.g., C_a, C_b update coefficients).
    pub materials: Vec<f64>, 

    pub time_step: usize,
}

impl SimulationState {
    pub fn new(width: usize, height: usize) -> Self {
        todo!("Initialize simulation state with zero fields")
    }

    pub fn reset(&mut self) {
        todo!("Reset all fields to zero")
    }
}
