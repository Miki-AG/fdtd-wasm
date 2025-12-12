import init, { FdtdSimulator } from '../pkg/fdtd_wasm.js';

let simulator = null;
let animationId = null;
let isRunning = false;

const canvas = document.getElementById('simCanvas');
const ctx = canvas.getContext('2d');
const signalCanvas = document.getElementById('signalCanvas');
const signalCtx = signalCanvas.getContext('2d');
const toggleBtn = document.getElementById('toggleBtn');
const resetBtn = document.getElementById('resetBtn');
const signalSelect = document.getElementById('signalType');
const scenarioSelect = document.getElementById('scenarioSelect');
const statsDiv = document.getElementById('stats');

const WIDTH = 1000;
const HEIGHT = 600;

canvas.width = WIDTH;
canvas.height = HEIGHT;

let currentScenarioConfig = null; 
let signalHistory = new Array(100).fill(0); // 100px wide

// ... createParabolaPath ... (unchanged)

function createParabolaPath(vertexX, vertexY, focalLength, height, openingRight = true) {
    // Equation: x = a*(y - k)^2 + h
    // a = 1 / (4 * f)
    // h = vertexX, k = vertexY
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

// ... getScenarioConfig ... (unchanged, just hiding to keep edit short if tool allows, but "replace" needs context. I will keep it all to be safe)

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
                createParabolaPath(smallVertexX, HEIGHT/2, smallFocalLen, 120, true),
                createParabolaPath(largeVertexX, HEIGHT/2, largeFocalLen, 400, false)
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
            obstacles: [] // No obstacles
        };
    } else {
        // Default Box
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
                `M ${WIDTH/2 - 25} ${HEIGHT/2 - 25} L ${WIDTH/2 + 25} ${HEIGHT/2 - 25} L ${WIDTH/2 + 25} ${HEIGHT/2 + 25} L ${WIDTH/2 - 25} ${HEIGHT/2 + 25} Z`
            ]
        };
    }
}

// ... rest ...

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

async function run() {
    await init();

    resetSimulation();

    toggleBtn.addEventListener('click', toggleSimulation);
    resetBtn.addEventListener('click', resetSimulation);
    signalSelect.addEventListener('change', resetSimulation);
    scenarioSelect.addEventListener('change', resetSimulation);
}

function resetSimulation() {
    stopSimulation();
    signalHistory.fill(0); // Clear history
    
    try {
        const config = getConfig();
        simulator = new FdtdSimulator(config);
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
        
        // Capture signal every step (or every N steps if too fast, but 5/frame is fine)
        if (currentScenarioConfig && currentScenarioConfig.receiver) {
            const val = simulator.get_field_at(currentScenarioConfig.receiver.x, currentScenarioConfig.receiver.y);
            signalHistory.push(val);
            signalHistory.shift();
        }
    }

    draw();
    drawSignal();
    updateStats();

    animationId = requestAnimationFrame(renderLoop);
}

function drawSignal() {
    signalCtx.fillStyle = '#000';
    signalCtx.fillRect(0, 0, 100, 100);
    
    signalCtx.beginPath();
    signalCtx.strokeStyle = '#0f0'; // Green scope trace
    signalCtx.lineWidth = 1;
    
    // Auto-scale or fixed? Fixed range [-10, 10] or relative to source?
    // Source amp is 50. Received will be much smaller.
    // Let's use a fixed zoom or simple scaling.
    const zoom = 5.0; // Zoom factor
    const midY = 50;
    
    for (let i = 0; i < 100; i++) {
        const val = signalHistory[i];
        // Plot
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

run();
