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
        const moduleName = path.basename(file, '.rs'); // e.g., 'parameters', 'state'

        // Regex to find 'pub fn'
        let fnMatch;
        const fnPattern = /pub fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(->\s*[^{]*)?/g;
        while ((fnMatch = fnPattern.exec(content)) !== null) {
            publicApis.push({
                module: moduleName,
                name: `${moduleName}::${fnMatch[1]}`,
                rawFnName: fnMatch[1],
                type: 'function', // Still capture type internally, but not display
                hasTest: false,
                status: 'No Test',
            });
        }

        // Regex to find methods in 'impl' blocks
        let implMatch;
        const implPattern = /impl\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\{([^}]*)\}/g;
        while ((implMatch = implPattern.exec(content)) !== null) {
            const structName = implMatch[1];
            const implBody = implMatch[2];

            let methodMatch;
            const methodPattern = /pub fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(([^)]*)\)\s*(->\s*[^{]*)?/g;
            while ((methodMatch = methodPattern.exec(implBody)) !== null) {
                publicApis.push({
                    module: moduleName,
                    name: `${structName}::${methodMatch[1]}`,
                    rawFnName: methodMatch[1],
                    type: 'method', // Still capture type internally
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

    // --- Correlate API with Tests ---
    for (const api of publicApis) {
        if (discoveredTests.some(testName => testName.includes(api.rawFnName))) {
            api.hasTest = true;
            api.status = 'Test Written (Status Unknown)';
        }
    }

    // --- Run cargo test ---
    let cargoTestOutput = '';
    let actualTestResults = [];
    try {
        console.log('Running cargo test to get actual results...');
        const result = execSync('cargo test --no-fail-fast --color never', { maxBuffer: 1024 * 1024 * 10, encoding: 'utf8' });
        cargoTestOutput = result;

        const testStatusPattern = /^test\s+([a-zA-Z_][a-zA-Z0-9_:]*)\s+\.\.\.\s+(ok|FAILED|ignored)/gm;
        let statusMatch;
        while ((statusMatch = testStatusPattern.exec(cargoTestOutput)) !== null) {
            actualTestResults.push({ name: statusMatch[1], status: statusMatch[2] });
        }
    } catch (e) {
        cargoTestOutput = (e.stdout || '') + (e.stderr || '');
        const testStatusPattern = /^test\s+([a-zA-Z_][a-zA-Z0-9_:]*)\s+\.\.\.\s+(ok|FAILED|ignored)/gm;
        let statusMatch;
        while ((statusMatch = testStatusPattern.exec(cargoTestOutput)) !== null) {
            actualTestResults.push({ name: statusMatch[1], status: statusMatch[2] });
        }
    }

    // Update Status
    for (const api of publicApis) {
        const matchingTest = actualTestResults.find(tr => tr.name.includes(api.rawFnName));
        if (matchingTest) {
            api.status = matchingTest.status === 'ok' ? 'Pass' : 'Fail';
        } else if (api.hasTest) {
             api.status = 'Test Written (Not Run/Wasm)';
        }
    }

    // --- GROUP BY MODULE ---
    const moduleOrder = ['parameters', 'state', 'rasterizer', 'renderer', 'engine', 'step', 'lib', 'utils'];
    
    // Create a map of Module -> APIs
    const groupedApis = {};
    moduleOrder.forEach(m => groupedApis[m] = []);
    publicApis.forEach(api => {
        if (!groupedApis[api.module]) {
            groupedApis[api.module] = [];
            if (!moduleOrder.includes(api.module)) {
                moduleOrder.push(api.module);
            }
        }
        groupedApis[api.module].push(api);
    });

    const totalApis = publicApis.length;
    const apisWithTests = publicApis.filter(api => api.hasTest).length;
    const apisWithoutTests = totalApis - apisWithTests;

    let flexItemsHtml = ''; // Change from tablesHtml to flexItemsHtml
    for (const mod of moduleOrder) {
        const apis = groupedApis[mod];
        if (!apis || apis.length === 0) continue;

        flexItemsHtml += `
            <div class="module-group">
                <h2>${mod.charAt(0).toUpperCase() + mod.slice(1)} Module</h2>
                <table>
                    <thead>
                        <tr>
                            <th>API Name</th>
                            <th>Test Status</th>
                        </tr>
                    </thead>
                    <tbody>
        `;
        
        apis.forEach(api => {
            flexItemsHtml += `
                        <tr>
                            <td>${api.name}</td>
                            <td class="status-${api.status.replace(/[\s\(\)\/]/g, '')}">${api.status}</td>
                        </tr>
            `;
        });

        flexItemsHtml += `
                    </tbody>
                </table>
            </div>
        `;
    }

    const htmlContent = `
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>API & Test Coverage Report</title>
    <style>
        body { font-family: sans-serif; font-size: 11px; padding: 10px; background-color: #f4f4f4; margin: 0; }
        h1 { font-size: 16px; margin: 10px 0; text-align: center; }
        h2 { font-size: 13px; margin-top: 5px; margin-bottom: 5px; color: #333; border-bottom: 1px solid #eee; padding-bottom: 3px; }
        .summary { font-size: 11px; margin-bottom: 10px; font-weight: bold; text-align: center; background: white; padding: 8px; border-radius: 4px; box-shadow: 0 1px 2px rgba(0,0,0,0.1); }
        
        /* Flexbox Layout */
        .flex-container {
            display: flex;
            flex-wrap: wrap;
            justify-content: space-between; /* Distributes items with space between them */
            gap: 10px; /* Space between flex items */
            margin-bottom: 10px;
        }
        
        .module-group { 
            background: white; 
            padding: 8px; 
            border-radius: 4px; 
            box-shadow: 0 1px 2px rgba(0,0,0,0.1); 
            flex: 1 1 calc(33.333% - 10px); /* Grow, shrink, base width for 3 columns with 10px gap */
            min-width: 250px; /* Minimum width to prevent items from becoming too narrow */
            box-sizing: border-box; /* Include padding and border in the element's total width and height */
        }

        /* Ensure "Discovered Tests" takes full width */
        .full-width-section {
            flex-basis: 100%; /* Take full width */
            margin-top: 10px; /* Add some space above */
        }

        table { border-collapse: collapse; width: 100%; }
        th, td { border-bottom: 1px solid #ddd; padding: 3px; text-align: left; }
        th { background-color: #f8f8f8; color: #555; }
        tr:last-child td { border-bottom: none; }
        
        .status-Pass { background-color: #dff0d8; color: #3c763d; font-weight: bold; }
        .status-Fail { background-color: #f2dede; color: #a94442; font-weight: bold; }
        .status-NoTest { background-color: #fcf8e3; color: #8a6d3b; }
        .status-TestWrittenNotRunWasm { background-color: #d9edf7; color: #31708f; }
        
        ul { columns: 3; -webkit-columns: 3; -moz-columns: 3; font-size: 10px; padding-left: 15px; margin: 0; }
        li { margin-bottom: 1px; }

        @media (max-width: 900px) {
            .module-group { flex: 1 1 calc(50% - 10px); } /* 2 columns */
            ul { columns: 2; -webkit-columns: 2; -moz-columns: 2; }
        }
        @media (max-width: 600px) {
            .module-group { flex: 1 1 100%; } /* 1 column */
            ul { columns: 1; -webkit-columns: 1; -moz-columns: 1; }
        }
    </style>
</head>
<body>
    <h1>API & Test Coverage Report</h1>
    <div class="summary">
        Date: ${new Date().toLocaleString()}<br>
        Total APIs: ${totalApis} | APIs with Tests: ${apisWithTests} | APIs without Tests: ${apisWithoutTests}
    </div>

    <div class="flex-container">
        ${flexItemsHtml}
    </div>

    <div class="module-group full-width-section">
        <h2>Discovered Tests</h2>
        <ul>
${discoveredTests.map(testName => `<li>${testName}</li>`).join('')}
        </ul>
    </div>
</body>
</html>
        `;

    const reportPath = path.join(process.cwd(), 'api_test_report.html');
    fs.writeFileSync(reportPath, htmlContent);

    console.log(`Comprehensive report generated: ${reportPath}`);
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