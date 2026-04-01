#!/bin/bash

# mailgen installation script
# Usage: curl -fsSL https://raw.githubusercontent.com/akin01/emailgen/main/install.sh | sudo bash

set -e

REPO="akin01/emailgen"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="mailgen"

# Detect OS
OS_TYPE="$(uname -s | tr '[:upper:]' '[:lower:]')"
case "${OS_TYPE}" in
    linux*)   PLATFORM="linux" ;;
    darwin*)  PLATFORM="macos" ;;
    msys*|cygwin*|mingw*) PLATFORM="windows" ;;
    *)        echo "Error: Unsupported OS: ${OS_TYPE}"; exit 1 ;;
esac

# Detect Architecture
ARCH_TYPE="$(uname -m)"
case "${ARCH_TYPE}" in
    x86_64|amd64) ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *)            echo "Error: Unsupported architecture: ${ARCH_TYPE}"; exit 1 ;;
esac

# Handle specific platform combinations
if [ "$PLATFORM" = "macos" ]; then
    ASSET_NAME="mailgen-macos-${ARCH}.tar.gz"
elif [ "$PLATFORM" = "linux" ]; then
    # We only build x86_64 for linux in the current CI
    if [ "$ARCH" != "x86_64" ]; then
        echo "Error: Linux ARM64 binaries are not currently provided. Please build from source."
        exit 1
    fi
    ASSET_NAME="mailgen-linux-x86_64.tar.gz"
elif [ "$PLATFORM" = "windows" ]; then
    ASSET_NAME="mailgen-windows-x86_64.zip"
fi

echo "Detecting latest version..."
LATEST_RELEASE=$(curl -s https://api.github.com/repos/${REPO}/releases/latest | grep "tag_name" | cut -d '"' -f 4)

if [ -z "$LATEST_RELEASE" ]; then
    echo "Error: Could not find latest release for ${REPO}"
    exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_RELEASE}/${ASSET_NAME}"

echo "Downloading mailgen ${LATEST_RELEASE} for ${PLATFORM}-${ARCH}..."
TEMP_DIR=$(mktemp -d)
curl -L "${DOWNLOAD_URL}" -o "${TEMP_DIR}/${ASSET_NAME}"

echo "Installing to ${INSTALL_DIR}..."
if [ "$PLATFORM" = "windows" ]; then
    unzip -q "${TEMP_DIR}/${ASSET_NAME}" -d "${TEMP_DIR}"
    mv "${TEMP_DIR}/mailgen.exe" "${INSTALL_DIR}/"
else
    # Extraction for Linux/macOS
    tar -xzf "${TEMP_DIR}/${ASSET_NAME}" -C "${TEMP_DIR}"
    mv "${TEMP_DIR}/mailgen" "${INSTALL_DIR}/"
    chmod +x "${INSTALL_DIR}/mailgen"
fi

rm -rf "${TEMP_DIR}"

echo "Successfully installed mailgen ${LATEST_RELEASE}!"
mailgen --version
