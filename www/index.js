import init, { FdtdSimulator } from '../pkg/fdtd_wasm.js';

let simulator = null;
let animationId = null;
let isRunning = false;

const canvas = document.getElementById('simCanvas');
const ctx = canvas.getContext('2d');
const toggleBtn = document.getElementById('toggleBtn');
const resetBtn = document.getElementById('resetBtn');
const statsDiv = document.getElementById('stats');

const WIDTH = 800;
const HEIGHT = 800;

canvas.width = WIDTH;
canvas.height = HEIGHT;

const defaultConfig = {
    width: WIDTH,
    height: HEIGHT,
    source: {
        x: WIDTH / 2,
        y: HEIGHT / 2,
        amplitude: 50.0, // High amplitude to see colors clearly with simple mapping
        frequency: 0.05
    },
    obstacles: [
        // A simple box
        "M 100 100 L 150 100 L 150 150 L 100 150 Z"
    ],
    duration_steps: 10000 // Must be > 0 to pass validation
};

async function run() {
    await init();

    resetSimulation();

    toggleBtn.addEventListener('click', toggleSimulation);
    resetBtn.addEventListener('click', resetSimulation);
}

function resetSimulation() {
    stopSimulation();

    // Create new simulator instance
    try {
        simulator = new FdtdSimulator(defaultConfig);
        draw(); // Draw initial state
        updateStats();
    } catch (e) {
        console.error("Failed to create simulator:", e);
        alert("Simulator init failed. Check console.");
    }
}

function toggleSimulation() {
    if (isRunning) {
        stopSimulation();
    } else {
        startSimulation();
    }
}

function startSimulation() {
    isRunning = true;
    toggleBtn.textContent = "Stop";
    renderLoop();
}

function stopSimulation() {
    isRunning = false;
    toggleBtn.textContent = "Start";
    if (animationId) {
        cancelAnimationFrame(animationId);
        animationId = null;
    }
}

function renderLoop() {
    if (!isRunning) return;

    // Run a few steps per frame for speed
    for (let i = 0; i < 5; i++) {
        simulator.step();
    }

    draw();
    updateStats();

    animationId = requestAnimationFrame(renderLoop);
}

function draw() {
    if (!simulator) return;

    const bufferPtr = simulator.get_frame_buffer(); // Returns Uint8Array (vec<u8> from rust)

    // In Rust we returned a Vec<u8>. wasm-bindgen converts this to a JS Uint8Array.
    // However, if we returned a *pointer* to memory, we'd need to view it.
    // Our current implementation `get_frame_buffer` returns `Vec<u8>`, which allocates and copies.
    // This is safe but slower. For now it's fine.

    const imageData = new ImageData(new Uint8ClampedArray(bufferPtr), WIDTH, HEIGHT);
    ctx.putImageData(imageData, 0, 0);
}

function updateStats() {
    if (simulator) {
        statsDiv.textContent = `Step: ${simulator.get_current_step()}`;
    }
}

run();
