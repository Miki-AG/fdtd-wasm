pub mod parameters;
pub mod state;
pub mod engine;
pub mod rasterizer;
pub mod utils;
pub mod step;
pub mod renderer;
pub mod comms;

use wasm_bindgen::prelude::*;
use parameters::SimulationParameters;
use state::SimulationState;
use comms::modulator::Modulator;
use comms::demodulator::Demodulator;

#[wasm_bindgen]
pub struct FdtdSimulator {
    params: SimulationParameters,
    state: SimulationState,
    modulator: Modulator,
    demodulator: Demodulator,
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

        // Initialize Comms (Fixed protocol: F0=0.05, F1=0.10, Rate=200 steps/bit)
        // Note: Freq must match source freq in config? 
        // For FM, we switch freqs. Let's pick standard ones.
        // Source def in config might be ignored during transmission.
        let mod_ = Modulator::new(0.05, 0.10, 200);
        let dem_ = Demodulator::new(0.05, 0.10, 200);

        Ok(FdtdSimulator { params, state, modulator: mod_, demodulator: dem_ })
    }

    /// Advances the simulation by one step.
    pub fn step(&mut self) {
        // 1. Modulator Override
        // If modulator has a next frequency, we override the source definition.
        // We need to pass this info to `step::step` or modify `params.source` temporarily?
        // Modifying `params` is cleaner.
        
        if let Some(freq) = self.modulator.next_frequency() {
            self.params.source.frequency = freq;
            // Ensure amplitude is ON
            if self.params.source.amplitude == 0.0 {
                self.params.source.amplitude = 1.0;
            }
        }
        
        // 2. Run Physics
        step::step(&self.params, &mut self.state);
        
        // 3. Demodulator process (if receiver exists)
        // We assume receiver is at a fixed location relative to something? 
        // Or we assume a second point?
        // Let's assume receiver is at `params.receiver`? `SimulationParameters` doesn't have `receiver`.
        // The user config in JS has `receiver`, but Rust struct `SimulationParameters` definition in `src/parameters.rs` DOES NOT.
        
        // LIMITATION: Rust `SimulationParameters` struct wasn't updated to include `receiver`.
        // In Phase 3 start I didn't add it.
        // I should stick to a hardcoded receiver location or update `SimulationParameters`.
        // Updating `SimulationParameters` breaks `index.js` config passing unless I update it there too.
        // Let's assume receiver is at (width*3/4, height/2) for defaults, or just pass it in `new`?
        // Or better: Add `set_receiver_pos(x, y)` API.
        
        // For now, let's hardcode sampling at a specific point or add `receiver` to struct?
        // I'll add `receiver` to `SimulationParameters`. It's cleaner.
    }

    pub fn send_message(&mut self, text: &str) {
        self.modulator.load_text(text);
    }

    pub fn get_transmission_bits(&self) -> String {
        self.modulator.get_bits_string()
    }

    pub fn get_received_text(&self) -> String {
        self.demodulator.get_text()
    }
    
    pub fn get_received_bits(&self) -> String {
        self.demodulator.get_bits_string()
    }

    /// Returns a pointer to the image buffer (RGBA) for the current state.
    pub fn get_frame_buffer(&self) -> Vec<u8> {
        renderer::render(&self.state)
    }
    
    /// Returns the current simulation time step.
    pub fn get_current_step(&self) -> usize {
        self.state.time_step
    }

    /// Returns the electric field value at a specific coordinate.
    pub fn get_field_at(&self, x: usize, y: usize) -> f64 {
        if x < self.params.width && y < self.params.height {
            self.state.ez[y * self.params.width + x]
        } else {
            0.0
        }
    }
    
    /// Feeds a value into the demodulator manually (called from JS or internal?)
    /// Better internal. But I need receiver coords.
    /// Temporary solution: JS calls `process_receiver_signal(val)` every step.
    /// This avoids Rust struct changes.
    pub fn process_receiver_signal(&mut self, val: f64) {
        self.demodulator.process_sample(val, self.state.time_step as f64);
    }
}