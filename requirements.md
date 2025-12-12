# Project Requirements: 2D FDTD Simulator (Wasm/Rust)

## 1. Project Overview
Build a high-performance 2D Finite-Difference Time-Domain (FDTD) electromagnetic simulator using Rust and WebAssembly (Wasm). The project aims to run simulations directly in a web browser with a focus on modular design and rigorous testing.

## 2. Core Technologies
- **Language:** Rust
- **Target Platform:** WebAssembly (Wasm)
- **Frontend:** HTML5 Canvas / JavaScript (for rendering and UI)

## 3. Functional Requirements

### 3.1. Simulation Inputs
The system must accept the following simulation parameters:
- **Simulation Space:** Dimensions ($W \times H$), e.g., $2000 \times 2000$ grid points.
- **Source:** 
    - Position ($x, y$)
    - Amplitude ($A$)
    - Frequency ($F$)
    - Waveform type (implicitly Sinusoidal based on Freq, but extensible).
- **Obstacles:**
    - Material: Metallic (Perfect Electric Conductor - PEC).
    - Definition: Array of shapes defined in **SVG path format**.
    - Processing: SVG paths must be rasterized onto the simulation grid (boolean mask: metal or free space).
- **Simulation Control:**
    - Duration (total time steps or physical time).

### 3.2. Simulation Core (FDTD)
- Implement standard 2D TMz (or TEz) FDTD update equations.
- Boundary Conditions: (To be determined, initially standard Absorbing Boundary Conditions or PEC walls).
- Grid update loop must be optimized for Wasm execution.

### 3.3. Visualization / Output
- **Target:** 2D HTML5 Canvas.
- **Color Mapping:**
    - Background/Zero: **Black**
    - Positive Field values: Gradient to **Red**
    - Negative Field values: Gradient to **Blue**
- **Obstacles:** Visually distinct (e.g., White or Gray overlay).

### 3.4. User Interface
- **Controls:**
    - [Start Simulation] Button.
- **Feedback:**
    - Step Counter / Progress indicator.

## 4. Development Roadmap

### Phase 1: Architecture & Design
- Define all data structures (Structs, Enums).
- Define the entire data pipeline.
- Write all function names and type signatures in Rust.
- **Deliverable:** Compilable Rust code with `todo!()` macros or empty bodies, defining the API surface. No logic implementation.

### Phase 2: Test-Driven Setup
- Create comprehensive test cases for all defined functions and modules.
- Implement a custom test runner or output formatter that generates an **HTML Report**.
- **Deliverable:** A test suite that runs and produces an HTML report showing **100% Failure** (Red).

### Phase 3: Implementation & Green Build
- Implement functions one by one.
- **Deliverable:** Gradual transition of the HTML report from Red to Green until 100% pass rate is achieved.
