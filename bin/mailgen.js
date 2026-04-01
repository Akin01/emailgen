#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const os = require('os');
const { spawnSync } = require('child_process');

const primaryBinName = os.platform() === 'win32' ? 'mailgen.exe' : 'mailgen';
const legacyBinName = os.platform() === 'win32' ? 'emailgen.exe' : 'emailgen';
const primaryBinPath = path.join(__dirname, primaryBinName);
const legacyBinPath = path.join(__dirname, legacyBinName);
const binPath = fs.existsSync(primaryBinPath) ? primaryBinPath : legacyBinPath;

if (!fs.existsSync(binPath)) {
    console.error(`mailgen binary not found. Looked for ${primaryBinName} and ${legacyBinName}. Please reinstall the package.`);
    process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: 'inherit'
});

process.exit(result.status);
