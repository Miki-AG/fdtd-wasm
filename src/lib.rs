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
use comms::modulator::{Modulator, ModulationScheme};
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
    // ... new ...

    /// Advances the simulation by one step.
    pub fn step(&mut self) {
        // 1. Modulator Override
        if let Some((freq, amp)) = self.modulator.next_modulation() {
            self.params.source.frequency = freq;
            self.params.source.amplitude = amp * 50.0; // Scale by base amplitude? 
            // Or assume base amplitude is 50.0? 
            // Better: use the amplitude from JS config as "Max Amplitude".
            // But I don't store "Max Amplitude". I store "Current Amplitude" in params.source.
            // Hack: Assume 50.0 is the "ON" level.
        }
        
        // 2. Run Physics
        step::step(&self.params, &mut self.state);
    }

    pub fn set_comms_scheme(&mut self, is_ask: bool) {
        let scheme = if is_ask { ModulationScheme::ASK } else { ModulationScheme::FSK };
        self.modulator.set_scheme(scheme);
        self.demodulator.set_scheme(scheme);
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

    pub fn get_demodulator_status(&self) -> String {
        self.demodulator.get_state_string()
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