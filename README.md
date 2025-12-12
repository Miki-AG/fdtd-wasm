# FDTD WASM Simulator

A high-performance 2D Finite-Difference Time-Domain (FDTD) electromagnetic simulation running in the browser using Rust and WebAssembly. This project visualizes wave propagation and simulates digital communication modulation schemes.

## Features

### Electromagnetic Simulation
*   **2D TMz FDTD Engine**: Simulates the propagation of electric ($E_z$) and magnetic ($H_x, H_y$) fields.
*   **Boundary Conditions**: Absorbing Boundary Conditions (ABC) to minimize edge reflections.
*   **Scenarios**:
    *   **Free Space**: Wave propagation without obstacles.
    *   **Double Parabolic Antenna**: Demonstrates focusing and directional transmission/reception.
    *   **Simple Box**: Interaction with basic geometry.

### Digital Communications System
This simulator includes a fully functional Modulator and Demodulator to demonstrate digital signal processing concepts directly within the physics simulation.

*   **Modulation Schemes**:
    *   **FSK (Frequency Shift Keying)**: Encodes bits by switching between two frequencies. Robust against amplitude noise.
    *   **ASK (Amplitude Shift Keying)**: Encodes bits by switching the amplitude. Simple but more susceptible to noise.
*   **Packet Structure**: Implements a robust packet format with Preamble, Sync Word, Length byte, Payload, and CRC-8 checksum.
*   **Text Transmission**: Send custom text messages through the simulated medium.
*   **Incremental Decoding**: Watch characters appear in real-time as packets are decoded (`T` -> `Te` -> `Tex` -> `Text`).

### Interactive Controls
*   **Signal Type**: Choose between Continuous (Sine/Square) or Pulsed sources for physics experiments.
*   **Comms Controls**:
    *   **Bit Rate Slider**: Adjust the transmission speed (Symbol Duration).
    *   **Noise Generator**: Inject thermal noise into the receiver to test robustness (SNR experiments).
    *   **Gain**: Amplify the received signal visualization.

## Setup & Build

### Prerequisites
*   [Rust](https://www.rust-lang.org/tools/install)
*   [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)

### Building
To compile the Rust code to WebAssembly for the browser:

```bash
wasm-pack build --target web
```

*Note: The `--target web` flag is essential for loading the modules directly in `index.html` without a bundler like Webpack.*

### Running
FDTD-WASM requires a local web server to serve the WASM files correctly (due to CORS and MIME types).

Using python:
```bash
python3 -m http.server
```
Then open `http://localhost:8000/www/` in your browser.

## Tech Stack
*   **Core**: Rust
*   **Wasm Interface**: `wasm-bindgen`
*   **Build Tool**: `wasm-pack`
*   **Frontend**: HTML5 Canvas, Vanilla JavaScript
*   **Serialization**: `serde`

## License
MIT