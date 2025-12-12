use crate::state::SimulationState;
use crate::parameters::{SourceDefinition, SignalType};

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
    let val = compute_source_signal(t, source.frequency, source.amplitude, &source.signal_type);
    let idx = source.y * state.width + source.x;
    if idx < state.ez.len() {
        state.ez[idx] += val; // Soft source
    }
}

/// Applies a specific/forced source value to the grid.
pub fn apply_forced_source(state: &mut SimulationState, x: usize, y: usize, value: f64) {
    let idx = y * state.width + x;
    if idx < state.ez.len() {
        state.ez[idx] += value;
    }
}

// ... boundaries unchanged ... (leaving this comment here is fine as I am replacing the blocks above/below boundaries if I match correctly, but I will match the exact blocks)

/// Applies Absorbing Boundary Conditions (ABC) to the Left boundary.
pub fn apply_boundary_left(state: &mut SimulationState) {
    let w = state.width;
    let h = state.height;
    let depth = 20.min(w / 2);
    
    for x in 0..depth {
        let factor = (x as f64 / depth as f64).powi(2); // Parabolic profile 0->1
        for y in 0..h {
            state.ez[y * w + x] *= factor;
            state.hx[y * w + x] *= factor;
            state.hy[y * w + x] *= factor;
        }
    }
}

/// Applies Absorbing Boundary Conditions (ABC) to the Right boundary.
pub fn apply_boundary_right(state: &mut SimulationState) {
    let w = state.width;
    let h = state.height;
    let depth = 20.min(w / 2);
    
    for x in 0..depth {
        let factor = (x as f64 / depth as f64).powi(2);
        let actual_x = w - 1 - x;
        for y in 0..h {
            state.ez[y * w + actual_x] *= factor;
            state.hx[y * w + actual_x] *= factor;
            state.hy[y * w + actual_x] *= factor;
        }
    }
}

/// Applies Absorbing Boundary Conditions (ABC) to the Top boundary.
pub fn apply_boundary_top(state: &mut SimulationState) {
    let w = state.width;
    let h = state.height;
    let depth = 20.min(h / 2);
    
    for y in 0..depth {
        let factor = (y as f64 / depth as f64).powi(2);
        for x in 0..w {
            state.ez[y * w + x] *= factor;
            state.hx[y * w + x] *= factor;
            state.hy[y * w + x] *= factor;
        }
    }
}

/// Applies Absorbing Boundary Conditions (ABC) to the Bottom boundary.
pub fn apply_boundary_bottom(state: &mut SimulationState) {
    let w = state.width;
    let h = state.height;
    let depth = 20.min(h / 2);
    
    for y in 0..depth {
        let factor = (y as f64 / depth as f64).powi(2);
        let actual_y = h - 1 - y;
        for x in 0..w {
            state.ez[actual_y * w + x] *= factor;
            state.hx[actual_y * w + x] *= factor;
            state.hy[actual_y * w + x] *= factor;
        }
    }
}

/// Helper to compute the signal value at a given time `t`.
pub fn compute_source_signal(t: f64, frequency: f64, amplitude: f64, signal_type: &SignalType) -> f64 {
    let omega = 2.0 * std::f64::consts::PI * frequency;
    match signal_type {
        SignalType::ContinuousSine => {
            amplitude * (omega * t).sin()
        },
        SignalType::ContinuousSquare => {
            amplitude * (omega * t).sin().signum()
        },
        SignalType::PulseSine => {
            // Single cycle sine wave
            let period = 1.0 / frequency;
            if t < period {
                amplitude * (omega * t).sin()
            } else {
                0.0
            }
        },
        SignalType::PulseSquare => {
            // Single square pulse (half period duration)
            let period = 1.0 / frequency;
            if t < period / 2.0 {
                amplitude
            } else if t < period {
                -amplitude
            } else {
                0.0
            }
        }
    }
}