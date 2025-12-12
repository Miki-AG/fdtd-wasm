use fdtd_wasm::renderer::{map_value_to_color, render};
use fdtd_wasm::state::SimulationState;

#[test]
fn test_map_value_to_color_zero() {
    let color = map_value_to_color(0.0);
    // Expect Black: [0, 0, 0, 255]
    assert_eq!(color, [0, 0, 0, 255]);
}

#[test]
fn test_map_value_to_color_positive_max() {
    let color = map_value_to_color(1.0);
    // Expect Red: [255, 0, 0, 255]
    assert_eq!(color, [255, 0, 0, 255]);
}

#[test]
fn test_map_value_to_color_negative_max() {
    let color = map_value_to_color(-1.0);
    // Expect Blue: [0, 0, 255, 255]
    assert_eq!(color, [0, 0, 255, 255]);
}

#[test]
fn test_render_output_size() {
    let state = SimulationState::new(10, 10); // 100 pixels
    let buffer = render(&state);
    // 4 bytes per pixel (RGBA)
    assert_eq!(buffer.len(), 10 * 10 * 4);
}
