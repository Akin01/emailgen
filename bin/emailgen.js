#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const os = require('os');
const { spawnSync } = require('child_process');

const binName = os.platform() === 'win32' ? 'emailgen.exe' : 'emailgen';
const binPath = path.join(__dirname, binName);

if (!fs.existsSync(binPath)) {
    console.error('emailgen binary not found. Please reinstall the package.');
    process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), {
    stdio: 'inherit'
});

process.exit(result.status);
