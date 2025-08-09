# Vāktra (वाक्त्र) Installation Script
# Automatically installs the Vāktra compiler with LLVM dependencies

param(
    [switch]$Force,
    [string]$InstallPath = "$env:USERPROFILE\.vaaktra"
)

Write-Host "🕉️  Installing Vāktra (वाक्त्र) - Sanskrit Programming Language" -ForegroundColor Cyan
Write-Host "=================================================" -ForegroundColor Cyan

# Check if Rust is installed
Write-Host "Checking Rust installation..." -ForegroundColor Yellow
try {
    $rustVersion = cargo --version
    Write-Host "✅ Rust found: $rustVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Rust not found. Please install Rust first:" -ForegroundColor Red
    Write-Host "   Visit: https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check if Git is installed
Write-Host "Checking Git installation..." -ForegroundColor Yellow
try {
    $gitVersion = git --version
    Write-Host "✅ Git found: $gitVersion" -ForegroundColor Green
} catch {
    Write-Host "❌ Git not found. Please install Git first:" -ForegroundColor Red
    Write-Host "   Visit: https://git-scm.com/download/win" -ForegroundColor Red
    exit 1
}

# Create installation directory
Write-Host "Creating installation directory..." -ForegroundColor Yellow
if (Test-Path $InstallPath) {
    if ($Force) {
        Remove-Item -Recurse -Force $InstallPath
    } else {
        Write-Host "❌ Installation directory already exists: $InstallPath" -ForegroundColor Red
        Write-Host "   Use -Force to overwrite" -ForegroundColor Red
        exit 1
    }
}
New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
Write-Host "✅ Installation directory created: $InstallPath" -ForegroundColor Green

# Clone the repository (or copy if running locally)
Write-Host "Setting up Vāktra source..." -ForegroundColor Yellow
if (Test-Path ".\Cargo.toml") {
    # Running from source directory
    Copy-Item -Recurse -Path "." -Destination $InstallPath -Exclude @(".git", "target", "*.log")
    Write-Host "✅ Copied source files to installation directory" -ForegroundColor Green
} else {
    # Clone from repository (when available)
    Write-Host "Cloning Vāktra repository..." -ForegroundColor Yellow
    git clone https://github.com/vaaktra/vaaktra.git $InstallPath
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Failed to clone repository" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Repository cloned successfully" -ForegroundColor Green
}

# Change to installation directory
Set-Location $InstallPath

# Check and install CMake if needed
Write-Host "Checking CMake installation..." -ForegroundColor Yellow
try {
    $cmakeVersion = cmake --version
    Write-Host "✅ CMake found: $($cmakeVersion.Split("`n")[0])" -ForegroundColor Green
} catch {
    Write-Host "⚠️  CMake not found, installing..." -ForegroundColor Yellow
    
    # Try to install CMake via Chocolatey
    try {
        choco install cmake -y
        
        # Refresh environment variables
        Write-Host "Refreshing environment variables..." -ForegroundColor Yellow
        Import-Module $env:ChocolateyInstall\helpers\chocolateyProfile.psm1 -ErrorAction SilentlyContinue
        refreshenv
        
        # Verify CMake is now available
        $cmakeVersion = cmake --version
        Write-Host "✅ CMake installed successfully: $($cmakeVersion.Split("`n")[0])" -ForegroundColor Green
    } catch {
        Write-Host "❌ Failed to install CMake automatically" -ForegroundColor Red
        Write-Host "Please install CMake manually:" -ForegroundColor Yellow
        Write-Host "  - Via Chocolatey: choco install cmake" -ForegroundColor White
        Write-Host "  - Via winget: winget install Kitware.CMake" -ForegroundColor White
        Write-Host "  - Download from: https://cmake.org/download/" -ForegroundColor White
        Write-Host "Then restart your terminal and run this script again." -ForegroundColor Yellow
        exit 1
    }
}

# Install llvmenv first
Write-Host "Installing llvmenv for LLVM management..." -ForegroundColor Yellow
cargo install llvmenv
if ($LASTEXITCODE -ne 0) {
    Write-Host "⚠️  Failed to install llvmenv, continuing with build script fallback" -ForegroundColor Yellow
} else {
    Write-Host "✅ llvmenv installed successfully" -ForegroundColor Green
}

# Build Vāktra compiler
Write-Host "Building Vāktra compiler..." -ForegroundColor Yellow
Write-Host "This may take a while as it includes LLVM setup..." -ForegroundColor Yellow

$env:RUST_LOG = "info"
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed. Check the output above for errors." -ForegroundColor Red
    Write-Host "The build script should have attempted to set up LLVM automatically." -ForegroundColor Yellow
    Write-Host "If LLVM setup failed, you may need to install LLVM manually." -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Vāktra compiler built successfully!" -ForegroundColor Green

# Add to PATH
Write-Host "Adding Vāktra to PATH..." -ForegroundColor Yellow
$binPath = Join-Path $InstallPath "target\release"
$currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")

if ($currentPath -notlike "*$binPath*") {
    $newPath = "$currentPath;$binPath"
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "✅ Added to PATH: $binPath" -ForegroundColor Green
    Write-Host "⚠️  Please restart your terminal to use the 'vaaktra' command" -ForegroundColor Yellow
} else {
    Write-Host "✅ Already in PATH" -ForegroundColor Green
}

# Test installation
Write-Host "Testing installation..." -ForegroundColor Yellow
$vaaktraPath = Join-Path $binPath "vaaktra.exe"
if (Test-Path $vaaktraPath) {
    Write-Host "✅ Vāktra executable found: $vaaktraPath" -ForegroundColor Green
    
    # Try to run it
    try {
        & $vaaktraPath --version
        Write-Host "✅ Vāktra is working correctly!" -ForegroundColor Green
    } catch {
        Write-Host "⚠️  Vāktra executable exists but may have runtime issues" -ForegroundColor Yellow
    }
} else {
    Write-Host "❌ Vāktra executable not found" -ForegroundColor Red
}

# Create desktop shortcut (optional)
Write-Host "Creating desktop shortcut..." -ForegroundColor Yellow
$desktopPath = [Environment]::GetFolderPath("Desktop")
$shortcutPath = Join-Path $desktopPath "Vāktra Compiler.lnk"

try {
    $WScriptShell = New-Object -ComObject WScript.Shell
    $shortcut = $WScriptShell.CreateShortcut($shortcutPath)
    $shortcut.TargetPath = "powershell.exe"
    $shortcut.Arguments = "-NoExit -Command `"cd '$InstallPath'; Write-Host 'Vāktra (वाक्त्र) Development Environment' -ForegroundColor Cyan`""
    $shortcut.WorkingDirectory = $InstallPath
    $shortcut.Description = "Vāktra Sanskrit Programming Language"
    $shortcut.Save()
    Write-Host "✅ Desktop shortcut created" -ForegroundColor Green
} catch {
    Write-Host "⚠️  Could not create desktop shortcut" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "🎉 Vāktra (वाक्त्र) Installation Complete!" -ForegroundColor Green
Write-Host "=================================================" -ForegroundColor Green
Write-Host "Installation Path: $InstallPath" -ForegroundColor Cyan
Write-Host "Executable: $vaaktraPath" -ForegroundColor Cyan
Write-Host ""
Write-Host "To get started:" -ForegroundColor Yellow
Write-Host "1. Restart your terminal (to refresh PATH)" -ForegroundColor White
Write-Host "2. Run: vaaktra --help" -ForegroundColor White
Write-Host "3. Create your first Sanskrit program!" -ForegroundColor White
Write-Host ""
Write-Host "Example usage:" -ForegroundColor Yellow
Write-Host "  vaaktra compile myprogram.vk" -ForegroundColor White
Write-Host "  vaaktra repl" -ForegroundColor White
Write-Host ""
Write-Host "Documentation: https://vaaktra.dev/docs" -ForegroundColor Cyan
Write-Host "Sanskrit Guide: https://vaaktra.dev/sanskrit" -ForegroundColor Cyan
Write-Host ""
Write-Host "नमस्ते! Welcome to Sanskrit programming! 🕉️" -ForegroundColor Magenta
