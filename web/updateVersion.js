const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const packageJsonPath = path.join(__dirname, 'package.json');
const jsFilePath = path.join(__dirname, 'src/layouts/verison.js');
const shortCommitHash = execSync('git rev-parse --short HEAD').toString().trim();
const currentDate = new Date().toISOString().replace(/T/, ' ').substr(0, 16);
// Read package.json
const packageJson = require(packageJsonPath);

// Update JavaScript content
let updatedContent = `export const version = '${packageJson.version}';`;
updatedContent += `\nexport const commitHash = '${shortCommitHash}';`;
updatedContent += `\nexport const buildDate = '${currentDate}';`;

// Write updated content back to the JavaScript file
fs.writeFile(jsFilePath, updatedContent, 'utf8', (writeErr) => {
    if (writeErr) {
        throw writeErr;
    } else {
        console.log(`VERSION: ${packageJson.version} ${shortCommitHash} ${currentDate}`);
    }
});
