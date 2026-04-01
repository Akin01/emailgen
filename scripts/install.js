const fs = require('fs');
const path = require('path');
const os = require('os');
const https = require('https');
const { execSync } = require('child_process');

const REPO = "akin01/emailgen";
const BIN_NAME = os.platform() === 'win32' ? 'mailgen.exe' : 'mailgen';
const DEST_DIR = path.join(__dirname, '..', 'bin');

if (!fs.existsSync(DEST_DIR)) {
    fs.mkdirSync(DEST_DIR, { recursive: true });
}

const getPlatformAsset = () => {
    const platform = os.platform();
    const arch = os.arch();

    if (platform === 'darwin') {
        if (arch === 'arm64') return `mailgen-macos-aarch64.tar.gz`;
        if (arch === 'x64') return `mailgen-macos-x86_64.tar.gz`;
    }
    if (platform === 'linux') {
        if (arch === 'x64') return `mailgen-linux-x86_64.tar.gz`;
    }
    if (platform === 'win32') {
        if (arch === 'x64') return `mailgen-windows-x86_64.zip`;
    }

    throw new Error(`Unsupported platform/architecture: ${platform}/${arch}`);
};

const download = (url, dest) => {
    return new Promise((resolve, reject) => {
        const file = fs.createWriteStream(dest);
        https.get(url, (response) => {
            if (response.statusCode === 302 || response.statusCode === 301) {
                download(response.headers.location, dest).then(resolve).catch(reject);
                return;
            }
            if (response.statusCode !== 200) {
                reject(new Error(`Download failed with status code ${response.statusCode}`));
                return;
            }
            response.pipe(file);
            file.on('finish', () => {
                file.close(resolve);
            });
        }).on('error', (err) => {
            fs.unlink(dest, () => {});
            reject(err);
        });
    });
};

const install = async () => {
    try {
        const assetName = getPlatformAsset();
        console.log(`Getting latest release for ${REPO}...`);
        
        const apiUrl = `https://api.github.com/repos/${REPO}/releases/latest`;
        const options = {
            headers: { 'User-Agent': 'node.js' }
        };

        const release = await new Promise((resolve, reject) => {
            https.get(apiUrl, options, (res) => {
                let data = '';
                res.on('data', chunk => data += chunk);
                res.on('end', () => resolve(JSON.parse(data)));
                res.on('error', reject);
            });
        });

        const tag = release.tag_name;
        const downloadUrl = `https://github.com/${REPO}/releases/download/${tag}/${assetName}`;
        const tempPath = path.join(os.tmpdir(), assetName);

        console.log(`Downloading ${assetName} from ${tag}...`);
        await download(downloadUrl, tempPath);

        console.log(`Extracting to ${DEST_DIR}...`);
        if (assetName.endsWith('.zip')) {
            const extractCmd = `powershell Expand-Archive -Path "${tempPath}" -DestinationPath "${DEST_DIR}" -Force`;
            execSync(extractCmd);
        } else {
            const extractCmd = `tar -xzf "${tempPath}" -C "${DEST_DIR}"`;
            execSync(extractCmd);
        }

        if (os.platform() !== 'win32') {
            fs.chmodSync(path.join(DEST_DIR, BIN_NAME), 0o755);
        }

        console.log(`Successfully installed mailgen!`);
        fs.unlinkSync(tempPath);
    } catch (err) {
        console.error(`Error during installation: ${err.message}`);
        process.exit(1);
    }
};

install();
