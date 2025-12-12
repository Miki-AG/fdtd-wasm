pub mod parameters;
pub mod state;
pub mod engine;
pub mod rasterizer;
pub mod utils;

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
        todo!("Parse config, initialize state, rasterize obstacles")
    }

    /// Advances the simulation by one step.
    pub fn step(&mut self) {
        todo!("Call engine update functions")
    }

    /// Returns a pointer to the image buffer (RGBA) for the current state.
    /// This is a simplified interface; in reality we might return a raw pointer or a Uint8ClampedArray.
    /// For Phase 1, we define the signature.
    pub fn get_frame_buffer(&self) -> Vec<u8> {
        todo!("Map Ez fields to colors (Black->Red/Blue) and return RGBA buffer")
    }
    
    /// Returns the current simulation time step.
    pub fn get_current_step(&self) -> usize {
        todo!("Return current step")
    }
}