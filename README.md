# Vāktra (वाक्त्र) Programming Language

A high-performance, statically-typed programming language based on Sanskrit, designed to be faster than C++ and Java while maintaining modern language features and safety guarantees.

## Features (लक्षणानि)

- **उच्च-प्रदर्शनम् (High Performance)**: Compiles to native code for maximum speed
- **सुरक्षितम् (Safe)**: Memory safety without garbage collection overhead
- **आधुनिकम् (Modern)**: Zero-cost abstractions and modern concurrency
- **संस्कृतम् (Sanskrit)**: Syntax and keywords based on Sanskrit language

## Example (उदाहरणम्)

```sanskrit
// प्रमुखं कार्यम् (Main function)
प्रधानं() {
    // संख्या (Integer)
    संख्या x = १०;
    
    // सत्यासत्य (Boolean)
    सत्यासत्य सत्य = सत्यम्;
    
    // पाठ (String)
    पाठ नमस्ते = "नमस्ते विश्व!";
    
    // चक्रीय (Loop)
    यावत् (x > ०) {
        मुद्रयतु(x);
        x = x - १;
    }
}
```

## Installation (स्थापना)

Vāktra provides automated installation scripts that handle all dependencies, including LLVM setup.

### Quick Install (त्वरित स्थापना)

**Windows (PowerShell):**
```powershell
# Download and run the installation script
iwr -useb https://install.vaaktra.dev/install.ps1 | iex

# Or if running locally:
.\install.ps1
```

**Linux/macOS (Bash):**
```bash
# Download and run the installation script
curl -sSf https://install.vaaktra.dev/install.sh | sh

# Or if running locally:
chmod +x install.sh && ./install.sh
```

### Manual Installation (स्वयं स्थापना)

#### Prerequisites (आवश्यकताएं)
- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)
- **Git** - For cloning the repository
- **LLVM** - Automatically handled by our build system

#### Build from Source (स्रोत से निर्माण)

1. **Clone the repository:**
   ```bash
   git clone https://github.com/vaaktra/vaaktra.git
   cd vaaktra
   ```

2. **Build with automatic LLVM setup:**
   ```bash
   cargo build --release
   ```
   
   The build script will automatically:
   - Install `llvmenv` if not present
   - Configure LLVM 15.0.0 for JIT compilation
   - Build all Vāktra components with Sanskrit integration

3. **Add to PATH:**
   ```bash
   # Linux/macOS
   export PATH="$PWD/target/release:$PATH"
   
   # Windows (PowerShell)
   $env:PATH += ";$PWD\target\release"
   ```

### Automated LLVM Setup (स्वचालित LLVM स्थापना)

Vāktra automatically manages LLVM dependencies through:

- **Build Script**: `build.rs` handles LLVM detection and setup
- **llvmenv Integration**: Automatically installs and configures LLVM 15.0.0
- **Fallback Mode**: Graceful degradation if LLVM setup fails
- **Cross-Platform**: Works on Windows, Linux, and macOS

The build process will:
1. Check for existing LLVM installations
2. Install `llvmenv` if needed
3. Configure LLVM 15.0.0 with optimized settings
4. Build the Vāktra compiler with JIT support

## Usage (उपयोग)

After installation, you can use Vāktra:

```bash
# Check version
vaaktra --version

# Compile a program
vaaktra compile program.vk

# Run REPL
vaaktra repl

# Get help
vaaktra --help
```

## Troubleshooting (समस्या निवारण)

### LLVM Setup Issues

If you encounter LLVM-related build errors, here are common solutions:

#### 1. llvmenv Configuration Issue
**Error**: `No entries. Please define entries in $XDG_CONFIG_HOME/llvmenv/entry.toml`

**Solution**: The `entry.toml` file needs to be in the correct location for your OS:
- **Windows**: `%APPDATA%\llvmenv\entry.toml` (e.g., `C:\Users\USERNAME\AppData\Roaming\llvmenv\entry.toml`)
- **Linux/macOS**: `~/.config/llvmenv/entry.toml`

Our build script handles this automatically, but if you're setting up manually:

```bash
# Create the directory
mkdir -p ~/.config/llvmenv  # Linux/macOS
mkdir "$env:APPDATA\llvmenv"  # Windows PowerShell

# Copy our provided configuration
cp llvmenv-config/entry.toml ~/.config/llvmenv/  # Linux/macOS
copy llvmenv-config\entry.toml "$env:APPDATA\llvmenv\"  # Windows
```

#### 2. CMake Not Found
**Error**: `CommandNotFound { cmd: "cmake" }`

**Solution**: Install CMake before building LLVM:

```bash
# Windows (Chocolatey)
choco install cmake

# Windows (winget)
winget install Kitware.CMake

# Ubuntu/Debian
sudo apt-get install cmake

# macOS (Homebrew)
brew install cmake

# CentOS/RHEL
sudo yum install cmake
```

**Important for Windows**: After installing CMake via Chocolatey, you may need to refresh your environment variables:

```powershell
# Import Chocolatey profile and refresh environment
Import-Module $env:ChocolateyInstall\helpers\chocolateyProfile.psm1
refreshenv

# Verify CMake is now available
cmake --version
```

Alternatively, restart your terminal/PowerShell session after CMake installation.

#### 3. LLVM Build Dependencies
For manual LLVM compilation, you may also need:
- **Windows**: Visual Studio Build Tools or Visual Studio Community
- **Linux**: `build-essential`, `clang`, `python3`
- **macOS**: Xcode Command Line Tools

#### 4. Environment Variables
If LLVM is installed but not detected, set the appropriate environment variable:

```bash
# For LLVM 15.0
export LLVM_SYS_150_PREFIX=/path/to/llvm  # Linux/macOS
$env:LLVM_SYS_150_PREFIX="C:\path\to\llvm"  # Windows PowerShell
```

### Build Performance Tips

- LLVM compilation can take 30-60 minutes depending on your system
- Use `cargo build --release --jobs 1` if you have limited RAM
- The build script provides fallback mode if LLVM setup fails

## Documentation (दस्तावेज़ीकरणम्)

Documentation will be available in both English and Sanskrit.

## License (अनुज्ञापत्रम्)

MIT License
