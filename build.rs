//! Build script for Vāktra (वाक्त्र) - Sanskrit Programming Language
//! 
//! This build script automatically handles LLVM installation and configuration
//! to ensure the compiler can be built without manual LLVM setup.

use std::env;
use std::process::Command;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Check if LLVM is already available
    if llvm_available() {
        println!("cargo:warning=LLVM found, proceeding with build");
        return;
    }
    
    println!("cargo:warning=LLVM not found, setting up automatic installation...");
    
    // Try to install and configure llvmenv + LLVM
    if let Err(e) = setup_llvm() {
        println!("cargo:warning=Failed to setup LLVM automatically: {}", e);
        println!("cargo:warning=Please install LLVM manually or use: cargo install llvmenv");
        
        // For now, we'll continue without LLVM to allow basic compilation
        // This allows the project to build in a degraded mode
        setup_fallback_mode();
    }
}

/// Check if LLVM is available in the system
fn llvm_available() -> bool {
    // Check for LLVM_SYS_*_PREFIX environment variables
    for (key, _) in env::vars() {
        if key.starts_with("LLVM_SYS_") && key.ends_with("_PREFIX") {
            return true;
        }
    }
    
    // Check for llvm-config in PATH
    if Command::new("llvm-config").arg("--version").output().is_ok() {
        return true;
    }
    
    // Check common LLVM installation paths on Windows
    let common_paths = [
        "C:\\Program Files\\LLVM",
        "C:\\Program Files (x86)\\LLVM",
        "C:\\LLVM",
    ];
    
    for path in &common_paths {
        if Path::new(path).exists() {
            // Try to set the environment variable for this build
            let version_paths = [
                format!("{}\\bin\\llvm-config.exe", path),
                format!("{}\\bin\\llvm-config", path),
            ];
            
            for config_path in &version_paths {
                if Path::new(config_path).exists() {
                    // Found LLVM, set environment variable
                    env::set_var("LLVM_SYS_150_PREFIX", path);
                    println!("cargo:rustc-env=LLVM_SYS_150_PREFIX={}", path);
                    return true;
                }
            }
        }
    }
    
    false
}

/// Setup LLVM using llvmenv
fn setup_llvm() -> Result<(), Box<dyn std::error::Error>> {
    // Check for CMake first
    if !cmake_available() {
        return Err("CMake is required for building LLVM. Please install CMake first.".into());
    }
    
    println!("cargo:warning=Installing llvmenv...");
    
    // Install llvmenv if not already installed
    let output = Command::new("cargo")
        .args(&["install", "llvmenv"])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("Failed to install llvmenv: {}", 
                          String::from_utf8_lossy(&output.stderr)).into());
    }
    
    println!("cargo:warning=Setting up LLVM 15.0 with llvmenv...");
    
    // Create llvmenv configuration
    setup_llvmenv_config()?;
    
    // Build LLVM 15.0
    let output = Command::new("llvmenv")
        .args(&["build-entry", "llvm-15"])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("Failed to build LLVM with llvmenv: {}", 
                          String::from_utf8_lossy(&output.stderr)).into());
    }
    
    // Set the global LLVM version
    let output = Command::new("llvmenv")
        .args(&["global", "llvm-15"])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("Failed to set global LLVM version: {}", 
                          String::from_utf8_lossy(&output.stderr)).into());
    }
    
    println!("cargo:warning=LLVM 15.0 setup completed successfully!");
    Ok(())
}

/// Check if CMake is available
fn cmake_available() -> bool {
    Command::new("cmake").arg("--version").output().is_ok()
}

/// Setup llvmenv configuration
fn setup_llvmenv_config() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    use std::io::Write;
    
    // Get config directory
    let config_dir = if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
        format!("{}/llvmenv", xdg_config)
    } else if let Ok(home) = env::var("HOME") {
        format!("{}/.config/llvmenv", home)
    } else if let Ok(userprofile) = env::var("USERPROFILE") {
        format!("{}/.config/llvmenv", userprofile)
    } else {
        return Err("Cannot determine config directory".into());
    };
    
    // Create config directory
    fs::create_dir_all(&config_dir)?;
    
    // Create entry.toml with LLVM 15.0 configuration
    let entry_toml = format!("{}/entry.toml", config_dir);
    let mut file = fs::File::create(&entry_toml)?;
    
    writeln!(file, r#"[llvm-15]
url = "https://github.com/llvm/llvm-project/releases/download/llvmorg-15.0.0/llvm-project-15.0.0.src.tar.xz"
name = "15.0.0"

[llvm-15.build]
targets = ["X86"]
build_type = "Release"
link_type = "static"
cmake_options = [
    "-DLLVM_ENABLE_PROJECTS=clang",
    "-DLLVM_TARGETS_TO_BUILD=X86",
    "-DLLVM_INCLUDE_EXAMPLES=OFF",
    "-DLLVM_INCLUDE_TESTS=OFF",
    "-DLLVM_INCLUDE_BENCHMARKS=OFF"
]"#)?;
    
    println!("cargo:warning=Created llvmenv configuration at {}", entry_toml);
    Ok(())
}

/// Setup fallback mode without LLVM for basic compilation
fn setup_fallback_mode() {
    println!("cargo:warning=Setting up fallback mode without LLVM");
    println!("cargo:rustc-cfg=feature=\"no-llvm\"");
    
    // Disable LLVM-dependent features
    println!("cargo:rustc-cfg=vaaktra_no_jit");
    
    // This allows the project to compile in a basic mode
    // Users can still use the lexer, parser, and other non-JIT features
}
