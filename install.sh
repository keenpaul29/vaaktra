#!/bin/bash
# Vāktra (वाक्त्र) Installation Script for Unix/Linux/macOS
# Automatically installs the Vāktra compiler with LLVM dependencies

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
INSTALL_PATH="${HOME}/.vaaktra"
FORCE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --force)
            FORCE=true
            shift
            ;;
        --install-path)
            INSTALL_PATH="$2"
            shift 2
            ;;
        --help)
            echo "Vāktra (वाक्त्र) Installation Script"
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --force           Overwrite existing installation"
            echo "  --install-path    Custom installation path (default: ~/.vaaktra)"
            echo "  --help           Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${CYAN}🕉️  Installing Vāktra (वाक्त्र) - Sanskrit Programming Language${NC}"
echo -e "${CYAN}=================================================${NC}"

# Check if Rust is installed
echo -e "${YELLOW}Checking Rust installation...${NC}"
if command -v cargo &> /dev/null; then
    RUST_VERSION=$(cargo --version)
    echo -e "${GREEN}✅ Rust found: $RUST_VERSION${NC}"
else
    echo -e "${RED}❌ Rust not found. Please install Rust first:${NC}"
    echo -e "${RED}   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

# Check if Git is installed
echo -e "${YELLOW}Checking Git installation...${NC}"
if command -v git &> /dev/null; then
    GIT_VERSION=$(git --version)
    echo -e "${GREEN}✅ Git found: $GIT_VERSION${NC}"
else
    echo -e "${RED}❌ Git not found. Please install Git first${NC}"
    exit 1
fi

# Check for build dependencies
echo -e "${YELLOW}Checking build dependencies...${NC}"
MISSING_DEPS=()

if ! command -v cmake &> /dev/null; then
    MISSING_DEPS+=("cmake")
fi

if ! command -v make &> /dev/null; then
    MISSING_DEPS+=("make")
fi

if ! command -v clang &> /dev/null && ! command -v gcc &> /dev/null; then
    MISSING_DEPS+=("clang or gcc")
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Missing build dependencies: ${MISSING_DEPS[*]}${NC}"
    echo -e "${YELLOW}The build script will attempt to handle LLVM automatically,${NC}"
    echo -e "${YELLOW}but you may need to install these dependencies manually.${NC}"
    
    # Provide installation hints based on OS
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo -e "${BLUE}On Ubuntu/Debian: sudo apt-get install cmake make clang${NC}"
        echo -e "${BLUE}On CentOS/RHEL: sudo yum install cmake make clang${NC}"
        echo -e "${BLUE}On Arch: sudo pacman -S cmake make clang${NC}"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo -e "${BLUE}On macOS: brew install cmake make${NC}"
        echo -e "${BLUE}Xcode command line tools should provide clang${NC}"
    fi
    
    read -p "Continue anyway? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo -e "${GREEN}✅ Build dependencies found${NC}"
fi

# Create installation directory
echo -e "${YELLOW}Creating installation directory...${NC}"
if [ -d "$INSTALL_PATH" ]; then
    if [ "$FORCE" = true ]; then
        rm -rf "$INSTALL_PATH"
    else
        echo -e "${RED}❌ Installation directory already exists: $INSTALL_PATH${NC}"
        echo -e "${RED}   Use --force to overwrite${NC}"
        exit 1
    fi
fi

mkdir -p "$INSTALL_PATH"
echo -e "${GREEN}✅ Installation directory created: $INSTALL_PATH${NC}"

# Clone or copy source
echo -e "${YELLOW}Setting up Vāktra source...${NC}"
if [ -f "./Cargo.toml" ]; then
    # Running from source directory
    cp -r . "$INSTALL_PATH"
    # Clean up
    rm -rf "$INSTALL_PATH/.git" "$INSTALL_PATH/target" "$INSTALL_PATH"/*.log 2>/dev/null || true
    echo -e "${GREEN}✅ Copied source files to installation directory${NC}"
else
    # Clone from repository (when available)
    echo -e "${YELLOW}Cloning Vāktra repository...${NC}"
    git clone https://github.com/vaaktra/vaaktra.git "$INSTALL_PATH"
    echo -e "${GREEN}✅ Repository cloned successfully${NC}"
fi

# Change to installation directory
cd "$INSTALL_PATH"

# Install llvmenv first
echo -e "${YELLOW}Installing llvmenv for LLVM management...${NC}"
if cargo install llvmenv; then
    echo -e "${GREEN}✅ llvmenv installed successfully${NC}"
else
    echo -e "${YELLOW}⚠️  Failed to install llvmenv, continuing with build script fallback${NC}"
fi

# Build Vāktra compiler
echo -e "${YELLOW}Building Vāktra compiler...${NC}"
echo -e "${YELLOW}This may take a while as it includes LLVM setup...${NC}"

export RUST_LOG=info

if cargo build --release; then
    echo -e "${GREEN}✅ Vāktra compiler built successfully!${NC}"
else
    echo -e "${RED}❌ Build failed. Check the output above for errors.${NC}"
    echo -e "${YELLOW}The build script should have attempted to set up LLVM automatically.${NC}"
    echo -e "${YELLOW}If LLVM setup failed, you may need to install LLVM manually.${NC}"
    exit 1
fi

# Add to PATH
echo -e "${YELLOW}Adding Vāktra to PATH...${NC}"
BIN_PATH="$INSTALL_PATH/target/release"
SHELL_RC=""

# Detect shell and appropriate RC file
if [ -n "$ZSH_VERSION" ]; then
    SHELL_RC="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ]; then
    if [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    else
        SHELL_RC="$HOME/.bash_profile"
    fi
else
    # Try to detect from $SHELL
    case "$SHELL" in
        */zsh)
            SHELL_RC="$HOME/.zshrc"
            ;;
        */bash)
            SHELL_RC="$HOME/.bashrc"
            ;;
        */fish)
            SHELL_RC="$HOME/.config/fish/config.fish"
            ;;
        *)
            SHELL_RC="$HOME/.profile"
            ;;
    esac
fi

# Add to PATH if not already there
if ! echo "$PATH" | grep -q "$BIN_PATH"; then
    echo "export PATH=\"$BIN_PATH:\$PATH\"" >> "$SHELL_RC"
    echo -e "${GREEN}✅ Added to PATH in $SHELL_RC${NC}"
    echo -e "${YELLOW}⚠️  Please restart your terminal or run: source $SHELL_RC${NC}"
else
    echo -e "${GREEN}✅ Already in PATH${NC}"
fi

# Test installation
echo -e "${YELLOW}Testing installation...${NC}"
VAAKTRA_PATH="$BIN_PATH/vaaktra"
if [ -f "$VAAKTRA_PATH" ]; then
    echo -e "${GREEN}✅ Vāktra executable found: $VAAKTRA_PATH${NC}"
    
    # Try to run it
    if "$VAAKTRA_PATH" --version &>/dev/null; then
        echo -e "${GREEN}✅ Vāktra is working correctly!${NC}"
    else
        echo -e "${YELLOW}⚠️  Vāktra executable exists but may have runtime issues${NC}"
    fi
else
    echo -e "${RED}❌ Vāktra executable not found${NC}"
fi

# Create a simple launcher script
echo -e "${YELLOW}Creating launcher script...${NC}"
LAUNCHER_PATH="$HOME/.local/bin/vaaktra"
mkdir -p "$(dirname "$LAUNCHER_PATH")"

cat > "$LAUNCHER_PATH" << EOF
#!/bin/bash
# Vāktra (वाक्त्र) Launcher Script
exec "$VAAKTRA_PATH" "\$@"
EOF

chmod +x "$LAUNCHER_PATH"
echo -e "${GREEN}✅ Launcher script created: $LAUNCHER_PATH${NC}"

echo ""
echo -e "${GREEN}🎉 Vāktra (वाक्त्र) Installation Complete!${NC}"
echo -e "${GREEN}=================================================${NC}"
echo -e "${CYAN}Installation Path: $INSTALL_PATH${NC}"
echo -e "${CYAN}Executable: $VAAKTRA_PATH${NC}"
echo ""
echo -e "${YELLOW}To get started:${NC}"
echo -e "${NC}1. Restart your terminal (to refresh PATH)${NC}"
echo -e "${NC}2. Run: vaaktra --help${NC}"
echo -e "${NC}3. Create your first Sanskrit program!${NC}"
echo ""
echo -e "${YELLOW}Example usage:${NC}"
echo -e "${NC}  vaaktra compile myprogram.vk${NC}"
echo -e "${NC}  vaaktra repl${NC}"
echo ""
echo -e "${CYAN}Documentation: https://vaaktra.dev/docs${NC}"
echo -e "${CYAN}Sanskrit Guide: https://vaaktra.dev/sanskrit${NC}"
echo ""
echo -e "${MAGENTA}नमस्ते! Welcome to Sanskrit programming! 🕉️${NC}"
