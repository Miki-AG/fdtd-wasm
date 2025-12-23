import init, { FdtdSimulator } from '../pkg/fdtd_wasm.js';

// --- Globals ---
let simulator = null;
let animationId = null;
let isRunning = false;
let idleCounter = 0;
let currentScenarioConfig = null;
let signalHistory = new Array(100).fill(0); // 100px wide

let WIDTH = 1000;
let HEIGHT = 600;

// --- DOM Elements ---
const canvas = document.getElementById('simCanvas');
const ctx = canvas.getContext('2d');
const signalCanvas = document.getElementById('signalCanvas');
const fftCanvas = document.getElementById('fftCanvas');
const statsDiv = document.getElementById('stats');

const params = {
    scenario: 'free_space',
    modulation: 'FSK',
    rate: 0,
    noise: 0,
    gain: 5,
    message: '',
    txBits: '',
    rxBits: '',
    rxText: '',
    packetState: 'Idle',
    rxSignalValue: 0,
    rxSpectrumPeak: 0,
    signalScale: 1,
    spectrumScale: 1
};

const txPane = new Tweakpane.Pane({ 
    container: document.getElementById('txPaneContainer'),
    title: 'Transmitter' 
});
txPane.addInput(params, 'message', { label: 'Message' });
txPane.addButton({ title: 'Send Message' }).on('click', () => {
    if (simulator && params.message) {
        simulator.send_message(params.message);
        params.txBits = simulator.get_transmission_bits();
        params.message = '';
        txPane.refresh();
        if (!isRunning) startSimulation();
    }
});
txPane.addMonitor(params, 'txBits', { label: 'Last Tx' });

const paramsPane = new Tweakpane.Pane({ 
    container: document.getElementById('paramsPaneContainer'),
    title: 'Global Settings' 
});
paramsPane.addInput(params, 'scenario', {
    label: 'Scenario',
    options: {
        'Free Space': 'free_space',
        'Simple Box': 'box',
        'Double Parabolic': 'double_parabola'
    }
}).on('change', () => resetSimulation());

paramsPane.addInput(params, 'modulation', {
    label: 'Modulation',
    options: { ASK: 'ASK', FSK: 'FSK' }
}).on('change', (ev) => {
    if (simulator) simulator.set_comms_scheme(ev.value === 'ASK');
});

paramsPane.addInput(params, 'rate', {
    label: 'Bit Rate',
    min: 0, max: 180, step: 1
}).on('change', (ev) => {
    if (simulator) {
        const samples = 200 - ev.value;
        simulator.set_symbol_duration(samples);
    }
});

paramsPane.addInput(params, 'noise', {
    label: 'Noise Level',
    min: 0, max: 0.5, step: 0.01
});

paramsPane.addInput(params, 'gain', { label: 'Visual Gain', min: 1, max: 100 });

const rxPane = new Tweakpane.Pane({ 
    container: document.getElementById('rxPaneContainer'),
    title: 'Receiver Status' 
});

rxPane.addSeparator();

const signalMonitor = rxPane.addMonitor(params, 'rxSignalValue', {
    label: 'Signal',
    view: 'graph',
    min: -1,
    max: 1,
    interval: 0
});
rxPane.addInput(params, 'signalScale', {
    label: 'Signal Gain',
    min: 0.1,
    max: 100
});

rxPane.addSeparator();

const spectrumMonitor = rxPane.addMonitor(params, 'rxSpectrumPeak', {
    label: 'Spectrum',
    view: 'graph',
    min: 0,
    max: 1,
    interval: 0
});
rxPane.addInput(params, 'spectrumScale', {
    label: 'Spec. Gain',
    min: 0.1,
    max: 100
});

const rxTextMonitor = rxPane.addMonitor(params, 'rxText', { label: 'Text' });
const rxBitsMonitor = rxPane.addMonitor(params, 'rxBits', { label: 'Bits' });
const rxStateMonitor = rxPane.addMonitor(params, 'packetState', { label: 'Status' });

canvas.width = WIDTH;
canvas.height = HEIGHT;

// --- Helper Functions ---

function createParabolaPath(vertexX, vertexY, focalLength, height, openingRight = true) {
    const a = 1.0 / (4.0 * focalLength);
    const direction = openingRight ? 1.0 : -1.0;

    let path = "";
    const segments = 80;
    const startY = vertexY - height / 2;
    const endY = vertexY + height / 2;

    const thickness = 2;

    for (let i = 0; i <= segments; i++) {
        const y = startY + (endY - startY) * (i / segments);
        const x = vertexX + direction * a * Math.pow(y - vertexY, 2);
        if (i === 0) path += `M ${x} ${y} `;
        else path += `L ${x} ${y} `;
    }

    for (let i = segments; i >= 0; i--) {
        const y = startY + (endY - startY) * (i / segments);
        const x = vertexX + direction * (a * Math.pow(y - vertexY, 2) + thickness);
        path += `L ${x} ${y} `;
    }

    path += "Z";
    return path;
}

function getScenarioConfig(type) {
    const freq = 0.05;

    if (type === 'double_parabola') {
        const smallVertexX = 100;
        const smallFocalLen = 10;
        const smallFocusX = smallVertexX + smallFocalLen;

        const largeVertexX = 800;
        const largeFocalLen = 60;
        const largeFocusX = largeVertexX - largeFocalLen;

        return {
            source: {
                x: smallFocusX,
                y: HEIGHT / 2,
                amplitude: 50.0,
                frequency: freq,
                signal_type: 'ContinuousSine'
            },
            receiver: {
                x: largeFocusX,
                y: HEIGHT / 2
            },
            obstacles: [
                createParabolaPath(smallVertexX, HEIGHT / 2, smallFocalLen, 120, true),
                createParabolaPath(largeVertexX, HEIGHT / 2, largeFocalLen, 400, false)
            ]
        };
    } else if (type === 'free_space') {
        return {
            source: {
                x: WIDTH / 4,
                y: HEIGHT / 2,
                amplitude: 50.0,
                frequency: freq,
                signal_type: 'ContinuousSine'
            },
            receiver: {
                x: WIDTH * 3 / 4,
                y: HEIGHT / 2
            },
            obstacles: []
        };
    } else {
        return {
            source: {
                x: WIDTH / 4,
                y: HEIGHT / 2,
                amplitude: 50.0,
                frequency: freq,
                signal_type: 'ContinuousSine'
            },
            receiver: {
                x: WIDTH * 3 / 4,
                y: HEIGHT / 2
            },
            obstacles: [
                `M ${WIDTH / 2 - 25} ${HEIGHT / 2 - 25} L ${WIDTH / 2 + 25} ${HEIGHT / 2 - 25} L ${WIDTH / 2 + 25} ${HEIGHT / 2 + 25} L ${WIDTH / 2 - 25} ${HEIGHT / 2 + 25} Z`
            ]
        };
    }
}

function getConfig() {
    currentScenarioConfig = getScenarioConfig(params.scenario);
    return {
        width: WIDTH,
        height: HEIGHT,
        source: currentScenarioConfig.source,
        obstacles: currentScenarioConfig.obstacles,
        duration_steps: 100000
    };
}

function updateCommsUI() {
    if (simulator) {
        params.rxBits = simulator.get_received_bits().slice(-64);
        const partial = simulator.get_received_partial_text();
        if (partial && partial.length > 0) {
            params.rxText = partial + "_";
        } else {
            params.rxText = simulator.get_received_text();
        }
        params.packetState = simulator.get_demodulator_status();

        rxTextMonitor.refresh();
        rxBitsMonitor.refresh();
        rxStateMonitor.refresh();
    }
}

// --- Helper Functions ---

function draw() {
    if (!simulator) return;

    const bufferPtr = simulator.get_frame_buffer();
    const imageData = new ImageData(new Uint8ClampedArray(bufferPtr), WIDTH, HEIGHT);
    ctx.putImageData(imageData, 0, 0);

    if (currentScenarioConfig) {
        ctx.beginPath();
        ctx.arc(currentScenarioConfig.source.x, currentScenarioConfig.source.y, 4, 0, 2 * Math.PI);
        ctx.fillStyle = 'white';
        ctx.fill();
        ctx.strokeStyle = 'red';
        ctx.lineWidth = 1;
        ctx.stroke();

        if (currentScenarioConfig.receiver) {
            ctx.beginPath();
            ctx.arc(currentScenarioConfig.receiver.x, currentScenarioConfig.receiver.y, 4, 0, 2 * Math.PI);
            ctx.fillStyle = 'white';
            ctx.fill();
            ctx.strokeStyle = 'blue';
            ctx.lineWidth = 1;
            ctx.stroke();
        }
    }
}

function updateStats() {
    if (simulator) {
        statsDiv.textContent = `Step: ${simulator.get_current_step()}`;
    }
}

// --- Simulation Logic ---

function resetSimulation() {
    stopSimulation();
    signalHistory.fill(0);

    try {
        const config = getConfig();
        simulator = new FdtdSimulator(config);
        simulator.set_comms_scheme(params.modulation === 'ASK');
        draw();
        updateStats();
    } catch (e) {
        console.error("Failed to create simulator:", e);
        alert("Simulator init failed. Check console.");
    }
}


function startSimulation() {
    isRunning = true;
    renderLoop();
}

function stopSimulation() {
    isRunning = false;
    if (animationId) {
        cancelAnimationFrame(animationId);
        animationId = null;
    }
}

function renderLoop() {
    if (!isRunning) return;

    for (let i = 0; i < 5; i++) {
        simulator.step();

        if (currentScenarioConfig && currentScenarioConfig.receiver) {
            let val = simulator.get_field_at(currentScenarioConfig.receiver.x, currentScenarioConfig.receiver.y);

            // Inject Noise
            const noiseLevel = params.noise;
            if (noiseLevel > 0) {
                // Random noise between -noiseLevel and +noiseLevel
                val += (Math.random() - 0.5) * 2 * noiseLevel;
            }

            signalHistory.push(val);
            signalHistory.shift();

            // Feed graphs (Scale acts as multiplier/Gain)
            params.rxSignalValue = val * params.signalScale;
            
            let max = 0;
            for(let j=0; j<signalHistory.length; j++) {
                const abs = Math.abs(signalHistory[j]);
                if (abs > max) max = abs;
            }
            params.rxSpectrumPeak = max * params.spectrumScale;

            signalMonitor.refresh();
            spectrumMonitor.refresh();

            simulator.process_receiver_signal(val);

            // Auto-pause logic: stop if idle for too long
            if (params.packetState === 'Idle') {
                idleCounter++;
            } else {
                idleCounter = 0;
            }
        }
    }

    if (idleCounter > 1000) { // About 200 frames at 5 steps per frame
        stopSimulation();
        idleCounter = 0;
    }

    draw();
    updateStats();
    updateCommsUI();

    animationId = requestAnimationFrame(renderLoop);
}

// --- Initialization ---

async function loadConfig() {
    try {
        const response = await fetch('./config.json');
        const config = await response.json();
        
        WIDTH = config.dimensions.width;
        HEIGHT = config.dimensions.height;
        
        params.scenario = config.simulation.scenario;
        params.modulation = config.simulation.modulation;
        params.rate = config.simulation.rate;
        params.noise = config.simulation.noise;
        params.gain = config.simulation.gain;
        params.signalScale = config.ui.signalScale;
        params.spectrumScale = config.ui.spectrumScale;
        
        canvas.width = WIDTH;
        canvas.height = HEIGHT;
    } catch (e) {
        console.warn("Could not load config.json, using hardcoded defaults.", e);
    }
}

async function run() {
    await init();
    await loadConfig();
    resetSimulation();
}

run();
