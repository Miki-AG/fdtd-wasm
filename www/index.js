import init, { FdtdSimulator } from '../pkg/fdtd_wasm.js';

// --- Globals ---
let simulator = null;
let animationId = null;
let isRunning = false;
let currentScenarioConfig = null;
let signalHistory = new Array(100).fill(0); // 100px wide

const WIDTH = 1000;
const HEIGHT = 600;

// --- DOM Elements ---
const canvas = document.getElementById('simCanvas');
const ctx = canvas.getContext('2d');
const signalCanvas = document.getElementById('signalCanvas');
const signalCtx = signalCanvas.getContext('2d');
const fftCanvas = document.getElementById('fftCanvas');
const fftCtx = fftCanvas.getContext('2d');
const toggleBtn = document.getElementById('toggleBtn');
const resetBtn = document.getElementById('resetBtn');
const signalSelect = document.getElementById('signalType');
const scenarioSelect = document.getElementById('scenarioSelect');
const statsDiv = document.getElementById('stats');
const gainSlider = document.getElementById('gainSlider');
const gainValueLabel = document.getElementById('gainValue');
const modulationSelect = document.getElementById('modulationSelect');
const rateSlider = document.getElementById('rateSlider');
const noiseSlider = document.getElementById('noiseSlider');
const msgInput = document.getElementById('msgInput');
const sendBtn = document.getElementById('sendBtn');
const txBitsSpan = document.getElementById('txBits');
const rxBitsSpan = document.getElementById('rxBits');
const rxTextSpan = document.getElementById('rxText');
const packetStateSpan = document.getElementById('packetState');

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
                signal_type: signalSelect.value
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
                signal_type: signalSelect.value
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
                signal_type: signalSelect.value
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
    currentScenarioConfig = getScenarioConfig(scenarioSelect.value);
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
        if (rxBitsSpan) rxBitsSpan.textContent = simulator.get_received_bits().slice(-64);
        if (rxTextSpan) {
            const partial = simulator.get_received_partial_text();
            // If we are in the middle of a packet (partial is not empty), show it.
            // Otherwise show the last confirmed full message.
            if (partial && partial.length > 0) {
                rxTextSpan.textContent = partial + "_"; // Add cursor to show it's active
            } else {
                rxTextSpan.textContent = simulator.get_received_text();
            }
        }
        if (packetStateSpan) packetStateSpan.textContent = simulator.get_demodulator_status();
    }
}

function computeDFT(signal) {
    const N = signal.length;
    const spectrum = new Array(N / 2).fill(0);

    for (let k = 0; k < N / 2; k++) {
        let re = 0;
        let im = 0;
        for (let n = 0; n < N; n++) {
            const theta = -2 * Math.PI * k * n / N;
            re += signal[n] * Math.cos(theta);
            im += signal[n] * Math.sin(theta);
        }
        // Magnitude
        spectrum[k] = Math.sqrt(re * re + im * im);
    }
    return spectrum;
}

function drawFFT() {
    if (!fftCtx) return;
    fftCtx.fillStyle = '#000';
    fftCtx.fillRect(0, 0, 100, 100);

    const spectrum = computeDFT(signalHistory);
    // Find max for scaling
    let maxMag = 0;
    for (let i = 0; i < spectrum.length; i++) {
        if (spectrum[i] > maxMag) maxMag = spectrum[i];
    }

    fftCtx.fillStyle = '#0ff'; // Cyan bars

    const barWidth = 100 / spectrum.length;

    for (let i = 0; i < spectrum.length; i++) {
        let height = 0;
        if (maxMag > 0.001) {
            height = (spectrum[i] / maxMag) * 90; // leave 10px headroom
        }

        const x = i * barWidth;
        const y = 100 - height;
        fftCtx.fillRect(x, y, barWidth - 1, height);
    }
}

function drawSignal() {
    signalCtx.fillStyle = '#000';
    signalCtx.fillRect(0, 0, 100, 100);

    signalCtx.beginPath();
    signalCtx.strokeStyle = '#0f0';
    signalCtx.lineWidth = 1;

    const zoom = parseFloat(gainSlider.value);
    const midY = 50;

    for (let i = 0; i < 100; i++) {
        const val = signalHistory[i];
        const y = midY - val * zoom;
        if (i === 0) signalCtx.moveTo(i, y);
        else signalCtx.lineTo(i, y);
    }
    signalCtx.stroke();
}

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
        simulator.set_comms_scheme(modulationSelect.value === 'ASK');
        draw();
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

    for (let i = 0; i < 5; i++) {
        simulator.step();

        if (currentScenarioConfig && currentScenarioConfig.receiver) {
            let val = simulator.get_field_at(currentScenarioConfig.receiver.x, currentScenarioConfig.receiver.y);

            // Inject Noise
            if (noiseSlider) {
                const noiseLevel = parseFloat(noiseSlider.value);
                if (noiseLevel > 0) {
                    // Random noise between -noiseLevel and +noiseLevel
                    val += (Math.random() - 0.5) * 2 * noiseLevel;
                }
            }

            signalHistory.push(val);
            signalHistory.shift();
            simulator.process_receiver_signal(val);
        }
    }

    draw();
    drawSignal();
    drawFFT();
    updateStats();
    updateCommsUI();

    animationId = requestAnimationFrame(renderLoop);
}

// --- Initialization ---

async function run() {
    await init();

    resetSimulation();

    toggleBtn.addEventListener('click', toggleSimulation);
    resetBtn.addEventListener('click', resetSimulation);
    signalSelect.addEventListener('change', resetSimulation);
    scenarioSelect.addEventListener('change', resetSimulation);

    modulationSelect.addEventListener('change', () => {
        if (simulator) {
            simulator.set_comms_scheme(modulationSelect.value === 'ASK');
        }
    });

    gainSlider.addEventListener('input', (e) => {
        gainValueLabel.textContent = `${e.target.value}x`;
    });

    rateSlider.addEventListener('input', (e) => {
        if (simulator) {
            const val = parseInt(e.target.value, 10);
            // Inverted logic: High Slider = High Rate = Low Duration
            // min(0) -> 200 samples. max(180) -> 20 samples.
            const samples = 200 - val;
            simulator.set_symbol_duration(samples);
        }
    });

    sendBtn.addEventListener('click', () => {
        console.log("Send clicked. Simulator:", !!simulator, "Msg:", msgInput.value);
        if (simulator && msgInput.value) {
            console.log("Sending message...");
            simulator.send_message(msgInput.value);
            const bits = simulator.get_transmission_bits();
            console.log("Bits:", bits);
            txBitsSpan.textContent = bits;
            msgInput.value = '';

            if (!isRunning) {
                console.log("Starting simulation...");
                startSimulation();
            } else {
                console.log("Simulation already running.");
            }
        }
    });
}

run();
