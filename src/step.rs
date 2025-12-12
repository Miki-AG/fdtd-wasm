use crate::parameters::SimulationParameters;
use crate::state::SimulationState;
use crate::engine::{
    update_hx, update_hy, update_e_fields, apply_source,
    apply_boundary_left, apply_boundary_right, apply_boundary_top, apply_boundary_bottom
};

/// Executes a single simulation step.
/// This orchestrates the field updates, source injection, and boundary conditions.
pub fn step(params: &SimulationParameters, state: &mut SimulationState) {
    // 1. Update Magnetic Fields (Hx, Hy)
    update_hx(state);
    update_hy(state);

    // 2. Update Electric Fields (Ez)
    update_e_fields(state);

    // 3. Apply Source
    apply_source(state, &params.source);

    // 4. Apply Boundary Conditions
    apply_boundary_left(state);
    apply_boundary_right(state);
    apply_boundary_top(state);
    apply_boundary_bottom(state);

    // 5. Advance Time
    state.time_step += 1;
}