use crate::state::SimulationState;
use crate::parameters::SourceDefinition;

/// Updates the magnetic field Hx for one time step.
pub fn update_hx(state: &mut SimulationState) {
    todo!("Implement Hx update equations")
}

/// Updates the magnetic field Hy for one time step.
pub fn update_hy(state: &mut SimulationState) {
    todo!("Implement Hy update equations")
}

/// Updates the electric field (Ez) for one time step.
/// This includes applying the source and boundary conditions.
pub fn update_e_fields(state: &mut SimulationState) {
    todo!("Implement E-field update equations")
}

/// Applies the source function to the grid.
pub fn apply_source(state: &mut SimulationState, source: &SourceDefinition) {
    todo!("Inject source signal into Ez field")
}

/// Applies Absorbing Boundary Conditions (ABC) to the Left boundary.
pub fn apply_boundary_left(state: &mut SimulationState) {
    todo!("Implement ABC for Left boundary")
}

/// Applies Absorbing Boundary Conditions (ABC) to the Right boundary.
pub fn apply_boundary_right(state: &mut SimulationState) {
    todo!("Implement ABC for Right boundary")
}

/// Applies Absorbing Boundary Conditions (ABC) to the Top boundary.
pub fn apply_boundary_top(state: &mut SimulationState) {
    todo!("Implement ABC for Top boundary")
}

/// Applies Absorbing Boundary Conditions (ABC) to the Bottom boundary.
pub fn apply_boundary_bottom(state: &mut SimulationState) {
    todo!("Implement ABC for Bottom boundary")
}

/// Helper to compute the signal value at a given time `t`.
pub fn compute_source_signal(t: f64, frequency: f64, amplitude: f64) -> f64 {
    todo!("Calculate A * sin(2 * pi * f * t)")
}