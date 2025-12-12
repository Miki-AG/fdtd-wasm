use crate::state::SimulationState;

/// Renders the current simulation state to an RGBA buffer.
pub fn render(state: &SimulationState) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(state.width * state.height * 4);
    for &val in &state.ez {
        let color = map_value_to_color(val);
        buffer.extend_from_slice(&color);
    }
    buffer
}

pub fn map_value_to_color(value: f64) -> [u8; 4] {
    // Basic mapping:
    // val > 0 -> Red intensity
    // val < 0 -> Blue intensity
    // val = 0 -> Black
    // Clamp at +/- 1.0 for max intensity
    
    let intensity = (value.abs().min(1.0) * 255.0) as u8;
    
    if value > 0.0 {
        [intensity, 0, 0, 255]
    } else if value < 0.0 {
        [0, 0, intensity, 255]
    } else {
        [0, 0, 0, 255]
    }
}
