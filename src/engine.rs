use crate::state::SimulationState;
use crate::parameters::SourceDefinition;

/// Updates the magnetic field Hx for one time step.
pub fn update_hx(state: &mut SimulationState) {
    let w = state.width;
    let h = state.height;
    // Hx(x, y) depends on Ez(x, y) and Ez(x, y+1)
    // Loop y from 0 to h-2
    for y in 0..h - 1 {
        for x in 0..w {
            let idx = y * w + x;
            let idx_up = (y + 1) * w + x;
            // Coefficient 0.5 is arbitrary for demo stability
            state.hx[idx] -= 0.5 * (state.ez[idx_up] - state.ez[idx]);
        }
    }
}

/// Updates the magnetic field Hy for one time step.
pub fn update_hy(state: &mut SimulationState) {
    let w = state.width;
    let h = state.height;
    // Hy(x, y) depends on Ez(x, y) and Ez(x+1, y)
    // Loop x from 0 to w-2
    for y in 0..h {
        for x in 0..w - 1 {
            let idx = y * w + x;
            let idx_right = y * w + (x + 1);
            state.hy[idx] += 0.5 * (state.ez[idx_right] - state.ez[idx]);
        }
    }
}

/// Updates the electric field (Ez) for one time step.
pub fn update_e_fields(state: &mut SimulationState) {
    let w = state.width;
    let h = state.height;
    // Ez(x, y) depends on Hy(x, y) - Hy(x-1, y) and Hx(x, y) - Hx(x, y-1)
    // Loop interior points
    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let idx = y * w + x;
            if state.materials[idx] > 0.0 {
                // Metal / Obstacle
                state.ez[idx] = 0.0;
                continue;
            }

            let idx_left = y * w + (x - 1);
            let idx_down = (y - 1) * w + x;
            
            // Standard FDTD update
            let dhy = state.hy[idx] - state.hy[idx_left];
            let dhx = state.hx[idx] - state.hx[idx_down];
            
            state.ez[idx] += 0.5 * (dhy - dhx);
        }
    }
}

/// Applies the source function to the grid.
pub fn apply_source(state: &mut SimulationState, source: &SourceDefinition) {
    let t = state.time_step as f64;
    let val = compute_source_signal(t, source.frequency, source.amplitude);
    let idx = source.y * state.width + source.x;
    if idx < state.ez.len() {
        state.ez[idx] += val; // Soft source
    }
}

/// Applies Absorbing Boundary Conditions (ABC) to the Left boundary.
pub fn apply_boundary_left(state: &mut SimulationState) {
    // Simple PEC (Perfect Electric Conductor) - Force 0
    let w = state.width;
    let h = state.height;
    for y in 0..h {
        state.ez[y * w] = 0.0;
    }
}

/// Applies Absorbing Boundary Conditions (ABC) to the Right boundary.
pub fn apply_boundary_right(state: &mut SimulationState) {
    let w = state.width;
    let h = state.height;
    for y in 0..h {
        state.ez[y * w + (w - 1)] = 0.0;
    }
}

/// Applies Absorbing Boundary Conditions (ABC) to the Top boundary.
pub fn apply_boundary_top(state: &mut SimulationState) {
    // Top corresponds to y = 0 or y = h-1 depending on coords.
    // Assuming y=0 is top in SVG coords usually? Or bottom?
    // Let's just do y=0 line.
    let w = state.width;
    for x in 0..w {
        state.ez[x] = 0.0;
    }
}

/// Applies Absorbing Boundary Conditions (ABC) to the Bottom boundary.
pub fn apply_boundary_bottom(state: &mut SimulationState) {
    let w = state.width;
    let h = state.height;
    let offset = (h - 1) * w;
    for x in 0..w {
        state.ez[offset + x] = 0.0;
    }
}

/// Helper to compute the signal value at a given time `t`.
pub fn compute_source_signal(t: f64, frequency: f64, amplitude: f64) -> f64 {
    amplitude * (2.0 * std::f64::consts::PI * frequency * t).sin()
}