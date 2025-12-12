pub mod parameters;
pub mod state;
pub mod engine;
pub mod rasterizer;
pub mod utils;
pub mod step;
pub mod renderer;

use wasm_bindgen::prelude::*;
use parameters::SimulationParameters;
use state::SimulationState;

#[wasm_bindgen]
pub struct FdtdSimulator {
    params: SimulationParameters,
    state: SimulationState,
}

#[wasm_bindgen]
impl FdtdSimulator {
    /// Creates a new simulator instance with the given JSON configuration.
    #[wasm_bindgen(constructor)]
    pub fn new(config_json: JsValue) -> Result<FdtdSimulator, JsValue> {
        utils::set_panic_hook();

        let params: SimulationParameters = serde_wasm_bindgen::from_value(config_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse config: {}", e)))?;

        parameters::validate_parameters(&params)
            .map_err(|e| JsValue::from_str(&format!("Invalid parameters: {}", e)))?;

        let mut state = SimulationState::new(params.width, params.height);

        // Rasterize obstacles
        state.materials = rasterizer::rasterize_obstacles(params.width, params.height, &params.obstacles);

        Ok(FdtdSimulator { params, state })
    }

    /// Advances the simulation by one step.
    pub fn step(&mut self) {
        step::step(&self.params, &mut self.state);
    }

    /// Returns a pointer to the image buffer (RGBA) for the current state.
    /// This is a simplified interface; in reality we might return a raw pointer or a Uint8ClampedArray.
    /// For Phase 1, we define the signature.
    pub fn get_frame_buffer(&self) -> Vec<u8> {
        renderer::render(&self.state)
    }
    
    /// Returns the current simulation time step.
    pub fn get_current_step(&self) -> usize {
        self.state.time_step
    }
}