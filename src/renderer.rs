use crate::state::SimulationState;

/// Renders the current simulation state to an RGBA buffer.
pub fn render(state: &SimulationState) -> Vec<u8> {
    let mut buffer = Vec::with_capacity(state.width * state.height * 4);
    for (i, &val) in state.ez.iter().enumerate() {
        let material_value = state.materials[i];
        let color = map_value_to_color(val, material_value);
        buffer.extend_from_slice(&color);
    }
    buffer
}

pub fn map_value_to_color(value: f64, material_value: f64) -> [u8; 4] {
    if material_value > 0.0 {
        // Render obstacles in green
        return [0, 255, 0, 255]; 
    }
    
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
