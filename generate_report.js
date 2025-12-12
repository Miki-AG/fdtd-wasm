const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function generateReport() {
    console.log('Generating comprehensive API and Test Coverage Report...');

    let publicApis = [];
    let discoveredTests = [];

    // --- Discover Public API Functions ---
    const srcDir = path.join(__dirname, 'src');
    const rustFiles = fs.readdirSync(srcDir).filter(file => file.endsWith('.rs'));

    for (const file of rustFiles) {
        const filePath = path.join(srcDir, file);
        const content = fs.readFileSync(filePath, 'utf8');

        // Regex to find 'pub fn'
        let fnMatch;
        // Captures function name and arguments
        const fnPattern = /pub fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(->\s*[^{]*)?/g;
        while ((fnMatch = fnPattern.exec(content)) !== null) {
            publicApis.push({
                name: `${path.basename(file, '.rs')}::${fnMatch[1]}`,
                rawFnName: fnMatch[1], // Store just the function name for easier matching
                type: 'function',
                hasTest: false, // Default
                status: 'No Test',
            });
        }

        // Regex to find methods in 'impl' blocks for structs annotated with #[wasm_bindgen]
        // This is a simplified regex, might need refinement for complex cases
        let implMatch;
        const implPattern = /impl\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{([^}]*)\}/g;
        while ((implMatch = implPattern.exec(content)) !== null) {
            const structName = implMatch[1];
            const implBody = implMatch[2];

            let methodMatch;
            const methodPattern = /pub fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(->\s*[^{]*)?/g;
            while ((methodMatch = methodPattern.exec(implBody)) !== null) {
                publicApis.push({
                    name: `${structName}::${methodMatch[1]}`,
                    rawFnName: methodMatch[1], // Store just the method name for easier matching
                    type: 'method',
                    hasTest: false,
                    status: 'No Test',
                });
            }
        }
    }

    // --- Discover Tests ---
    const testsDir = path.join(__dirname, 'tests');
    const testFiles = fs.readdirSync(testsDir).filter(file => file.endsWith('.rs'));

    for (const file of testFiles) {
        const filePath = path.join(testsDir, file);
        const content = fs.readFileSync(filePath, 'utf8');

        let testMatch;
        const testPattern = /#\[test\]\s*fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(\)\s*\{/g;
        while ((testMatch = testPattern.exec(content)) !== null) {
            discoveredTests.push(testMatch[1]);
        }
    }

    // --- Correlate API with Tests (Revised Heuristic) ---
    for (const api of publicApis) {
        // Check if any test name contains the raw function/method name
        if (discoveredTests.some(testName => testName.includes(api.rawFnName))) {
            api.hasTest = true;
            api.status = 'Test Written (Status Unknown)'; // We don't run tests yet to determine pass/fail
        }
    }

    // --- Run cargo test to get actual results ---
    let cargoTestOutput = '';
    let actualTestResults = [];
    try {
        console.log('Running cargo test to get actual results...');
        // Use --no-fail-fast to ensure all tests are attempted
        const result = execSync('cargo test --no-fail-fast --color never', { maxBuffer: 1024 * 1024 * 10, encoding: 'utf8' });
        cargoTestOutput = result;

        const testStatusPattern = /^test\s+([a-zA-Z_][a-zA-Z0-9_:]*)\s+\.\.\.\s+(ok|FAILED|ignored)/gm;
        let statusMatch;
        while ((statusMatch = testStatusPattern.exec(cargoTestOutput)) !== null) {
            actualTestResults.push({ name: statusMatch[1], status: statusMatch[2] });
        }
    } catch (e) {
        cargoTestOutput = (e.stdout || '') + (e.stderr || ''); // Ensure stdout/stderr are strings
        // Still try to parse results even if cargo test exited with an error (e.g., due to failures)
        const testStatusPattern = /^test\s+([a-zA-Z_][a-zA-Z0-9_:]*)\s+\.\.\.\s+(ok|FAILED|ignored)/gm;
        let statusMatch;
        while ((statusMatch = testStatusPattern.exec(cargoTestOutput)) !== null) {
            actualTestResults.push({ name: statusMatch[1], status: statusMatch[2] });
        }
    }
    
    // Update API status with actual test results
    for (const api of publicApis) {
        const matchingTest = actualTestResults.find(tr => tr.name.includes(api.rawFnName));
        if (matchingTest) {
            api.status = matchingTest.status === 'ok' ? 'Pass' : 'Fail';
        } else if (api.hasTest) {
             // If a test was found by name, but not in actualTestResults, it likely means cargo test
             // didn't run it (e.g., wasm-bindgen issues for non-wasm target).
             api.status = 'Test Written (Not Run/Wasm)';
        }
    }

    const totalApis = publicApis.length;
    const apisWithTests = publicApis.filter(api => api.hasTest).length;
    const apisWithoutTests = totalApis - apisWithTests;

    const htmlContent = `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>API & Test Coverage Report</title>
    <style>
        body { font-family: sans-serif; padding: 20px; }
        h1 { margin-bottom: 10px; }
        .summary { margin-bottom: 20px; font-weight: bold; }
        .pass { color: green; }
        .fail { color: red; }
        .untested { color: orange; }
        table { border-collapse: collapse; width: 100%; max-width: 1000px; margin-top: 20px; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        tr:nth-child(even) { background-color: #f9f9f9; }
        .status-Pass { background-color: #dff0d8; color: #3c763d; }
        .status-Fail { background-color: #f2dede; color: #a94442; }
        .status-NoTest { background-color: #fcf8e3; color: #8a6d3b; }
        .status-TestWrittenNotRunWasm { background-color: #d9edf7; color: #31708f; } /* Blue for WASM-related not run */
    </style>
</head>
<body>
    <h1>API & Test Coverage Report</h1>
    <div class="summary">
        Date: ${new Date().toLocaleString()}<br>
        Total APIs: ${totalApis} | APIs with Tests: ${apisWithTests} | APIs without Tests: ${apisWithoutTests}
    </div>

    <h2>API Functions & Methods Overview</h2>
    <table>
        <thead>
            <tr>
                <th>API Name</th>
                <th>Type</th>
                <th>Test Status</th>
            </tr>
        </thead>
        <tbody>
${publicApis.map(api => `
            <tr>
                <td>${api.name}</td>
                <td>${api.type}</td>
                <td class="status-${api.status.replace(/[\s\(\)\/]/g, '')}">${api.status}</td>
            </tr>`).join('')}
        </tbody>
    </table>

    <h2>Discovered Tests</h2>
    <ul>
${discoveredTests.map(testName => `<li>${testName}</li>`).join('')}
    </ul>
    
    <h2>Raw Cargo Test Output</h2>
    <pre>${escapeHtml(cargoTestOutput)}</pre>
</body>
</html>
        `;

    const reportPath = path.join(process.cwd(), 'api_test_report.html');
    fs.writeFileSync(reportPath, htmlContent);

    console.log(`Comprehensive report generated: ${reportPath}`);
    console.log(`Summary: Total APIs=${totalApis}, APIs with Tests=${apisWithTests}, APIs without Tests=${apisWithoutTests}`);
}

function escapeHtml(text) {
    if (text === null || text === undefined) return '';
    return text
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#039;");
}

generateReport();