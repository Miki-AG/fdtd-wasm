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

// --- Physical Constants ---
let C_MS = 343.0; // Speed of sound in air (m/s)
let DX_MM = 1.0;  // 1 pixel = 1 mm
let DT = 0;       // Seconds per step (calculated)
let isQueueFrozen = false;

// --- DOM Elements ---
const canvas = document.getElementById('simCanvas');
const ctx = canvas.getContext('2d');
const signalCanvas = document.getElementById('signalCanvas');
const fftCanvas = document.getElementById('fftCanvas');
const statsDiv = document.getElementById('stats');
const queueBody = document.getElementById('queueBody');
const receptorQueueBody = document.getElementById('receptorQueueBody');

const params = {
    scenario: 'free_space',
    modulation: 'FSK',
    carrier_hz: 40000,
    dev_hz: 5000,
    baud: 1200,
    power: 1.0,
    noise: 0.02,
    gain: 10,
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
        prepareEmitterQueue(params.message);
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
}).on('change', () => resetSimulation());

paramsPane.addInput(params, 'carrier_hz', {
    label: 'Carrier (Hz)',
    min: 10000, max: 150000, step: 100
}).on('change', () => resetSimulation());

paramsPane.addInput(params, 'dev_hz', {
    label: 'Deviation (Hz)',
    min: 1000, max: 50000, step: 100
}).on('change', () => resetSimulation());

paramsPane.addInput(params, 'baud', {
    label: 'Baud Rate (bps)',
    min: 300, max: 9600, step: 100
}).on('change', () => resetSimulation());

paramsPane.addInput(params, 'power', {
    label: 'Power (W)',
    min: 0.1, max: 100, step: 0.1
}).on('change', () => resetSimulation());

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

function prepareEmitterQueue(text) {
    if (!queueBody) return;
    queueBody.innerHTML = '';
    
    const bytes = [];
    bytes.push({ label: 'PRE', val: 0xAA });
    bytes.push({ label: 'SYNC', val: 0x7E });
    
    const len = Math.min(text.length, 255);
    bytes.push({ label: 'LEN', val: len });
    
    let sum = 0;
    for (let i = 0; i < len; i++) {
        const charCode = text.charCodeAt(i);
        bytes.push({ label: text[i], val: charCode });
        sum += charCode;
    }
    
    bytes.push({ label: 'CRC', val: sum % 256 });
    
    bytes.forEach((byte, byteIdx) => {
        const tr = document.createElement('tr');
        const tdCmd = document.createElement('td');
        tdCmd.className = 'col-cmd';
        tdCmd.textContent = byte.label;
        
        const tdBin = document.createElement('td');
        tdBin.className = 'col-bin';
        
        const binStr = byte.val.toString(2).padStart(8, '0');
        for (let i = 0; i < 8; i++) {
            const span = document.createElement('span');
            span.className = 'bit';
            span.id = `bit-${byteIdx * 8 + i}`;
            span.textContent = binStr[i];
            tdBin.appendChild(span);
        }
        
        tr.appendChild(tdCmd);
        tr.appendChild(tdBin);
        queueBody.appendChild(tr);
    });
}

function updateQueueView() {
    if (!simulator || !queueBody) return;
    const bitIdx = simulator.get_transmission_bit_idx();
    const bits = queueBody.querySelectorAll('.bit');
    
    // Opt-out if index is beyond bits (idle)
    if (bitIdx > bits.length) return;

    for (let i = 0; i < bits.length; i++) {
        if (i < bitIdx) {
            bits[i].className = 'bit done';
        } else if (i === bitIdx) {
            bits[i].className = 'bit active';
        } else {
            bits[i].className = 'bit';
        }
    }
}

function updateReceptorQueue() {
    if (!simulator || !receptorQueueBody || isQueueFrozen) return;
    
    const history = simulator.get_receiver_history();
    const currentBits = simulator.get_receiver_current_bits();
    
    // Clear and rebuild (simple for now, can be optimized)
    receptorQueueBody.innerHTML = '';
    
    // Finished events
    history.forEach(event => {
        const tr = document.createElement('tr');
        
        // Coloring logic
        if (event.is_error) {
            tr.className = 'row-error';
        } else if (event.is_complete) {
            tr.className = 'row-success';
        }

        if (event.label === 'CRC') {
            isQueueFrozen = true;
        }

        const tdBin = document.createElement('td');
        tdBin.className = 'col-bin-rx';
        tdBin.textContent = event.bits;
        
        const tdCmd = document.createElement('td');
        tdCmd.className = 'col-cmd-rx';
        tdCmd.textContent = event.label;
        
        tr.appendChild(tdBin);
        tr.appendChild(tdCmd);
        receptorQueueBody.appendChild(tr);
    });
    
    // In-progress row
    if (currentBits.length > 0) {
        const tr = document.createElement('tr');
        const tdBin = document.createElement('td');
        tdBin.className = 'col-bin-rx';
        tdBin.textContent = currentBits;
        
        const tdCmd = document.createElement('td');
        tdCmd.className = 'col-cmd-rx';
        tdCmd.textContent = "...";
        
        tr.appendChild(tdBin);
        tr.appendChild(tdCmd);
        receptorQueueBody.appendChild(tr);
    }

    // Auto-scroll to bottom
    const container = document.getElementById('receptorQueueContainer');
    if (container) {
        container.scrollTop = container.scrollHeight;
    }
}

function resetReceptorQueue() {
    isQueueFrozen = false;
    if (receptorQueueBody) receptorQueueBody.innerHTML = '';
    if (simulator) simulator.reset_receiver();
}

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
    // FDTD Normalization
    const dt = (DX_MM / 1000.0) / (C_MS * Math.sqrt(2));
    const freqNorm = params.carrier_hz * dt;
    const amp = 50.0 * Math.sqrt(params.power);

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
                amplitude: amp,
                frequency: freqNorm,
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
                amplitude: amp,
                frequency: freqNorm,
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
                amplitude: amp,
                frequency: freqNorm,
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
    
    // Calculate DT based on current physics
    DT = (DX_MM / 1000.0) / (C_MS * Math.sqrt(2));
    
    const carrierNorm = params.carrier_hz * DT;
    const devNorm = params.dev_hz * DT;
    const durationSteps = Math.floor((1.0 / params.baud) / DT);

    return {
        width: WIDTH,
        height: HEIGHT,
        source: currentScenarioConfig.source,
        comms: {
            carrier_frequency: carrierNorm,
            deviation: devNorm,
            symbol_duration: durationSteps
        },
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
    if (simulator && currentScenarioConfig) {
        const step = simulator.get_current_step();
        const timeMs = (step * DT * 1000).toFixed(2);
        
        let distMsg = "";
        if (currentScenarioConfig.receiver) {
            const dx_pixels = currentScenarioConfig.receiver.x - currentScenarioConfig.source.x;
            const dy_pixels = currentScenarioConfig.receiver.y - currentScenarioConfig.source.y;
            const pixDist = Math.sqrt(dx_pixels**2 + dy_pixels**2);
            const metersDist = (pixDist * DX_MM / 1000.0).toFixed(2);
            distMsg = ` | Dist: ${metersDist}m`;
        }
        
        statsDiv.textContent = `Step: ${step} | Time: ${timeMs}ms${distMsg}`;
    }
}

// --- Simulation Logic ---

function resetSimulation() {
    stopSimulation();
    signalHistory.fill(0);
    resetReceptorQueue();

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
    updateQueueView();
    updateReceptorQueue();

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
        params.noise = config.simulation.noise;
        params.gain = config.simulation.gain;
        
        params.modulation = config.transmission.modulation;
        params.carrier_hz = config.transmission.carrier_hz;
        params.dev_hz = config.transmission.dev_hz;
        params.baud = config.transmission.baud;
        params.power = config.transmission.power_w;
        
        C_MS = config.physics.c_ms;
        DX_MM = config.physics.dx_mm;
        
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
