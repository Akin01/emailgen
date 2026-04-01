# mailgen Installation Script for Windows
# Usage: Invoke-WebRequest -Uri https://raw.githubusercontent.com/akin01/emailgen/main/install.ps1 -OutFile install.ps1; .\install.ps1
# Or: powershell -ExecutionPolicy Bypass -Command "Invoke-Expression (New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/akin01/emailgen/main/install.ps1')"

param(
    [string]$InstallDir = "$env:USERPROFILE\.cargo\bin",
    [switch]$Force
)

$ErrorActionPreference = "Stop"

# mailgen Installation Script for Windows
$REPO = "akin01/emailgen"
$BINARY_NAME = "mailgen.exe"

Write-Host "=== mailgen Windows Installer ===" -ForegroundColor Cyan

# Detect Architecture
$ARCH = $env:PROCESSOR_ARCHITECTURE
if ($ARCH -eq "AMD64") {
    $ARCH = "x86_64"
} elseif ($ARCH -eq "ARM64") {
    $ARCH = "aarch64"
} else {
    Write-Host "Error: Unsupported architecture: $ARCH" -ForegroundColor Red
    exit 1
}

Write-Host "Detected architecture: $ARCH" -ForegroundColor Green

# Check if Windows
if ($env:OS -ne "Windows_NT") {
    Write-Host "Error: This script is for Windows only. Use install.sh for Linux/macOS." -ForegroundColor Red
    exit 1
}

# Create install directory if it doesn't exist
if (-not (Test-Path $InstallDir)) {
    Write-Host "Creating install directory: $InstallDir" -ForegroundColor Yellow
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

# Check if mailgen is already installed
$ExistingBinary = Join-Path $InstallDir $BINARY_NAME
if (Test-Path $ExistingBinary) {
    if (-not $Force) {
        Write-Host "Warning: mailgen is already installed at $ExistingBinary" -ForegroundColor Yellow
        Write-Host "Use -Force to overwrite existing installation." -ForegroundColor Yellow
        $response = Read-Host "Do you want to overwrite? [y/N]"
        if ($response -ne "y" -and $response -ne "Y") {
            Write-Host "Installation cancelled." -ForegroundColor Yellow
            exit 0
        }
    }
    Write-Host "Removing existing installation..." -ForegroundColor Yellow
    Remove-Item -Path $ExistingBinary -Force
}

Write-Host "Detecting latest version..." -ForegroundColor Cyan
try {
    $LATEST_RELEASE = Invoke-RestMethod -Uri "https://api.github.com/repos/$REPO/releases/latest" -UseBasicParsing
    $VERSION = $LATEST_RELEASE.tag_name -replace "^v", ""
    Write-Host "Latest version: $VERSION" -ForegroundColor Green
} catch {
    Write-Host "Error: Could not fetch latest release information" -ForegroundColor Red
    Write-Host "Details: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

$ASSET_NAME = "mailgen-windows-$ARCH.zip"
$DOWNLOAD_URL = "https://github.com/$REPO/releases/download/v$VERSION/$ASSET_NAME"

Write-Host "Downloading mailgen $VERSION for Windows-$ARCH..." -ForegroundColor Cyan

$TEMP_DIR = [System.IO.Path]::GetTempPath() + "mailgen-install-" + [System.Guid]::NewGuid().ToString()
New-Item -ItemType Directory -Force -Path $TEMP_DIR | Out-Null

try {
    $DOWNLOAD_PATH = Join-Path $TEMP_DIR $ASSET_NAME
    
    # Download with progress
    $ProgressPreference = 'SilentlyContinue'
    Invoke-WebRequest -Uri $DOWNLOAD_URL -OutFile $DOWNLOAD_PATH -UseBasicParsing
    $ProgressPreference = 'Continue'
    
    Write-Host "Download complete!" -ForegroundColor Green
    
    # Extract
    Write-Host "Extracting..." -ForegroundColor Cyan
    Expand-Archive -Path $DOWNLOAD_PATH -DestinationPath $TEMP_DIR -Force
    
    # Find the binary (might be in a subdirectory)
    $ExtractedBinary = Get-ChildItem -Path $TEMP_DIR -Filter $BINARY_NAME -Recurse | Select-Object -First 1
    
    if (-not $ExtractedBinary) {
        Write-Host "Error: Could not find mailgen.exe in the downloaded archive" -ForegroundColor Red
        exit 1
    }
    
    # Install
    Write-Host "Installing to $InstallDir..." -ForegroundColor Cyan
    Copy-Item -Path $ExtractedBinary.FullName -Destination $ExistingBinary -Force
    
    # Verify installation
    if (Test-Path $ExistingBinary) {
        Write-Host "Installation successful!" -ForegroundColor Green
        Write-Host "Binary location: $ExistingBinary" -ForegroundColor Green
    } else {
        Write-Host "Error: Installation failed - binary not found at expected location" -ForegroundColor Red
        exit 1
    }
    
    # Check if install directory is in PATH
    $PATH = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($PATH -notlike "*$InstallDir*") {
        Write-Host "`nWarning: $InstallDir is not in your PATH" -ForegroundColor Yellow
        Write-Host "To use mailgen from anywhere, add this directory to your PATH:" -ForegroundColor Yellow
        Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$InstallDir', 'User')" -ForegroundColor Yellow
        Write-Host "`nOr restart your terminal after adding it manually." -ForegroundColor Yellow
        
        $response = Read-Host "Do you want to add it to PATH now? [y/N]"
        if ($response -eq "y" -or $response -eq "Y") {
            try {
                $NewPath = $PATH + ";$InstallDir"
                [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
                Write-Host "Added to PATH! Please restart your terminal to use mailgen." -ForegroundColor Green
                $env:Path = [Environment]::GetEnvironmentVariable("Path", "User")
            } catch {
                Write-Host "Error: Failed to update PATH" -ForegroundColor Red
                Write-Host "Please add it manually." -ForegroundColor Yellow
            }
        }
    }
    
    # Try to run mailgen --version if PATH was updated or already available
    if ($PATH -like "*$InstallDir*" -or $env:Path -like "*$InstallDir*") {
        Write-Host "`nVerifying installation..." -ForegroundColor Cyan
        try {
            & mailgen --version
        } catch {
            Write-Host "Note: Run 'mailgen --help' to get started!" -ForegroundColor Yellow
        }
    } else {
        Write-Host "`nTo verify installation, run:" -ForegroundColor Cyan
        Write-Host "  & '$ExistingBinary' --version" -ForegroundColor Yellow
    }
    
    Write-Host "`n=== Installation Complete ===" -ForegroundColor Green
    Write-Host "Quick start:" -ForegroundColor Cyan
    Write-Host "  mailgen --count 1000 --output emails.txt  # Generate 1000 emails" -ForegroundColor White
    Write-Host "  mailgen --help                             # Show all options" -ForegroundColor White
    
} catch {
    Write-Host "`nError: Installation failed" -ForegroundColor Red
    Write-Host "Details: $($_.Exception.Message)" -ForegroundColor Red
    
    # Cleanup
    if (Test-Path $TEMP_DIR) {
        Write-Host "Cleaning up temporary files..." -ForegroundColor Yellow
        Remove-Item -Path $TEMP_DIR -Recurse -Force
    }
    
    exit 1
} finally {
    # Cleanup temporary files
    if (Test-Path $TEMP_DIR) {
        Remove-Item -Path $TEMP_DIR -Recurse -Force
    }
}
