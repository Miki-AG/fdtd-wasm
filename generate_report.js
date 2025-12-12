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
                type: 'function',
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
    // Explicitly distribute modules into 3 columns
    const columns = [
        ['parameters', 'state', 'rasterizer'], // Column 1
        ['renderer', 'engine', 'step'],       // Column 2
        ['lib', 'utils']                      // Column 3
    ];
    
    // Create a map of Module -> APIs
    const groupedApis = {};
    columns.flat().forEach(m => groupedApis[m] = []);
    
    // Handle any extra modules not in list by adding them to the last column
    publicApis.forEach(api => {
        if (!groupedApis[api.module]) {
            groupedApis[api.module] = [];
            if (!columns.flat().includes(api.module)) {
                columns[2].push(api.module);
            }
        }
        groupedApis[api.module].push(api);
    });

    const totalApis = publicApis.length;
    const apisWithTests = publicApis.filter(api => api.hasTest).length;
    const apisWithoutTests = totalApis - apisWithTests;

    let columnsHtml = '';
    
    columns.forEach(moduleList => {
        let colContent = '<div class="column">';
        moduleList.forEach(mod => {
            const apis = groupedApis[mod];
            if (!apis || apis.length === 0) return;

            colContent += `
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
                colContent += `
                            <tr>
                                <td>${api.name}</td>
                                <td class="status-${api.status.replace(/[\s\(\)\/]/g, '')}">${api.status}</td>
                            </tr>
                `;
            });

            colContent += `
                        </tbody>
                    </table>
                </div>
            `;
        });
        colContent += '</div>';
        columnsHtml += colContent;
    });

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
        
        /* Column Container */
        .columns-container {
            display: flex;
            gap: 10px;
            align-items: flex-start; /* Prevent stretching height */
        }
        
        .column {
            flex: 1;
            display: flex;
            flex-direction: column;
            gap: 10px;
            min-width: 0; /* Allow flex shrinking */
        }

        .module-group { 
            background: white; 
            padding: 8px; 
            border-radius: 4px; 
            box-shadow: 0 1px 2px rgba(0,0,0,0.1); 
        }

        .full-width-section {
            margin-top: 10px;
        }

        table { border-collapse: collapse; width: 100%; table-layout: fixed; }
        th, td { border-bottom: 1px solid #ddd; padding: 3px; text-align: left; word-wrap: break-word; }
        th { background-color: #f8f8f8; color: #555; }
        tr:last-child td { border-bottom: none; }
        
        .status-Pass { background-color: #dff0d8; color: #3c763d; font-weight: bold; }
        .status-Fail { background-color: #f2dede; color: #a94442; font-weight: bold; }
        .status-NoTest { background-color: #fcf8e3; color: #8a6d3b; }
        .status-TestWrittenNotRunWasm { background-color: #d9edf7; color: #31708f; }
        
        ul { columns: 3; -webkit-columns: 3; -moz-columns: 3; font-size: 10px; padding-left: 15px; margin: 0; }
        li { margin-bottom: 1px; }

        @media (max-width: 700px) {
            .columns-container { flex-direction: column; }
            ul { columns: 1; }
        }
    </style>
</head>
<body>
    <h1>API & Test Coverage Report</h1>
    <div class="summary">
        Date: ${new Date().toLocaleString()}<br>
        Total APIs: ${totalApis} | APIs with Tests: ${apisWithTests} | APIs without Tests: ${apisWithoutTests}
    </div>

    <div class="columns-container">
        ${columnsHtml}
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
