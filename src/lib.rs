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
    is_transmitting: bool,
}

#[wasm_bindgen]
impl FdtdSimulator {
    #[wasm_bindgen(constructor)]
    pub fn new(val: JsValue) -> Result<FdtdSimulator, JsValue> {
        utils::set_panic_hook();
        let params: SimulationParameters = serde_wasm_bindgen::from_value(val)?;
        let width = params.width;
        let height = params.height;
        let freq = params.source.frequency;
        
        // FSK Parameters
        let samples_per_symbol = 200; 
        let freq_1 = freq;
        let freq_0 = freq * 0.5; 

        Ok(FdtdSimulator {
            params,
            state: SimulationState::new(width, height),
            modulator: Modulator::new(freq_0, freq_1, samples_per_symbol),
            demodulator: Demodulator::new(freq_0, freq_1, samples_per_symbol),
            is_transmitting: false,
        })
    }

    /// Advances the simulation by one step.
    pub fn step(&mut self) {
        let mut forced_source = None;

        // 1. Modulator Override if transmitting
        if self.is_transmitting {
            if let Some((freq, amp_factor)) = self.modulator.next_modulation() {
                 // Use a fixed max amplitude of 50.0 for now, or derive from existing params if we want to be fancy.
                 // The modulator returns amp_factor (0.0 or 1.0) for ASK, or 1.0 for FSK.
                 let max_amplitude = 50.0; 
                 let amplitude = amp_factor * max_amplitude;
                 
                 // We need to compute the instantaneous value of the signal (Sine wave)
                 // SignalType::ContinuousSine is hardcoded here for data transmission.
                 let t = self.state.time_step as f64;
                 let val = engine::compute_source_signal(t, freq, amplitude, &parameters::SignalType::ContinuousSine);
                 
                 forced_source = Some(val);
            } else {
                // Transmission done
                self.is_transmitting = false;
            }
        }
        
        // 2. Run Physics
        step::step(&self.params, &mut self.state, forced_source);
    }

    pub fn set_comms_scheme(&mut self, is_ask: bool) {
        let scheme = if is_ask { ModulationScheme::ASK } else { ModulationScheme::FSK };
        self.modulator.set_scheme(scheme);
        self.demodulator.set_scheme(scheme);
    }

    pub fn send_message(&mut self, text: &str) {
        self.modulator.load_text(text);
        self.is_transmitting = true;
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