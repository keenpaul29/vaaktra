//! Virtual Machine for Vāktra (वाक्त्र)
//! 
//! High-performance virtual machine with advanced execution strategies
//! inspired by Vedic concepts of consciousness (चेतना) and manifestation (अभिव्यक्ति).

pub mod bytecode;
pub mod interpreter;
pub mod stack;
pub mod gc;

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use thiserror::Error;

use vaaktra_parser::ast::Program;
use vaaktra_jit::runtime::{RuntimeValue, VaaktraRuntime};

/// VM execution errors
#[derive(Debug, Error)]
pub enum VmError {
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Stack overflow: {0}")]
    StackOverflow(String),
    
    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(String),
    
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
}

pub type VmResult<T> = Result<T, VmError>;

/// High-performance virtual machine for Vāktra
pub struct VaaktraVm {
    /// Bytecode interpreter
    interpreter: interpreter::BytecodeInterpreter,
    
    /// Execution stack
    stack: stack::VmStack,
    
    /// Garbage collector
    gc: gc::GarbageCollector,
    
    /// Global variables
    globals: Arc<RwLock<HashMap<String, RuntimeValue>>>,
    
    /// Runtime environment
    runtime: VaaktraRuntime,
    
    /// VM statistics
    stats: VmStats,
    
    /// Configuration
    config: VmConfig,
}

/// VM execution statistics
#[derive(Debug, Default)]
pub struct VmStats {
    pub instructions_executed: u64,
    pub function_calls: u64,
    pub gc_collections: u64,
    pub memory_allocated: u64,
    pub execution_time: std::time::Duration,
}

/// VM configuration
#[derive(Debug)]
pub struct VmConfig {
    /// Stack size in bytes
    pub stack_size: usize,
    
    /// Enable JIT compilation
    pub enable_jit: bool,
    
    /// JIT threshold (number of calls before JIT compilation)
    pub jit_threshold: u32,
    
    /// Enable garbage collection
    pub enable_gc: bool,
    
    /// GC threshold
    pub gc_threshold: usize,
    
    /// Enable profiling
    pub enable_profiling: bool,
}

impl VaaktraVm {
    /// Create a new virtual machine
    pub fn new() -> VmResult<Self> {
        Ok(VaaktraVm {
            interpreter: interpreter::BytecodeInterpreter::new()?,
            stack: stack::VmStack::new(1024 * 1024)?, // 1MB stack
            gc: gc::GarbageCollector::new()?,
            globals: Arc::new(RwLock::new(HashMap::new())),
            runtime: VaaktraRuntime::new()
                .map_err(|e| VmError::RuntimeError(e.to_string()))?,
            stats: VmStats::default(),
            config: VmConfig {
                stack_size: 1024 * 1024,
                enable_jit: true,
                jit_threshold: 100,
                enable_gc: true,
                gc_threshold: 1000,
                enable_profiling: false,
            },
        })
    }
    
    /// Execute a Vāktra program
    pub fn execute_program(&mut self, program: &Program) -> VmResult<RuntimeValue> {
        log::info!("Starting VM execution of Vāktra program");
        let start_time = std::time::Instant::now();
        
        // Compile program to bytecode
        let bytecode = self.interpreter.compile_program(program)?;
        
        // Execute bytecode
        let result = self.interpreter.execute(&bytecode, &mut self.stack, &self.globals)?;
        
        // Update statistics
        self.stats.execution_time += start_time.elapsed();
        
        log::info!("VM execution completed successfully");
        Ok(result)
    }
    
    /// Execute a single function
    pub fn execute_function(&mut self, name: &str, args: &[RuntimeValue]) -> VmResult<RuntimeValue> {
        let start_time = std::time::Instant::now();
        
        // Look up function in globals
        let globals = self.globals.read();
        if let Some(RuntimeValue::Mantra(func)) = globals.get(name) {
            drop(globals);
            let result = func(args)
                .map_err(|e| VmError::ExecutionError(e.to_string()))?;
            
            self.stats.function_calls += 1;
            self.stats.execution_time += start_time.elapsed();
            
            Ok(result)
        } else {
            Err(VmError::ExecutionError(format!("Function {} not found", name)))
        }
    }
    
    /// Register a function in the VM
    pub fn register_function(&mut self, name: String, func: fn(&[RuntimeValue]) -> Result<RuntimeValue, vaaktra_jit::runtime::RuntimeError>) {
        let mut globals = self.globals.write();
        globals.insert(name, RuntimeValue::Mantra(func));
    }
    
    /// Get VM statistics
    pub fn get_stats(&self) -> &VmStats {
        &self.stats
    }
    
    /// Configure the VM
    pub fn configure(&mut self, config: VmConfig) {
        self.config = config;
    }
    
    /// Trigger garbage collection
    pub fn collect_garbage(&mut self) -> VmResult<usize> {
        if !self.config.enable_gc {
            return Ok(0);
        }
        
        let collected = self.gc.collect(&self.stack, &self.globals)?;
        self.stats.gc_collections += 1;
        
        Ok(collected)
    }
    
    /// Get memory usage
    pub fn memory_usage(&self) -> usize {
        self.stack.size() + self.gc.heap_size()
    }
    
    /// Shutdown the VM
    pub fn shutdown(&mut self) -> VmResult<()> {
        log::info!("Shutting down Vāktra VM");
        
        // Clear globals
        self.globals.write().clear();
        
        // Reset stack
        self.stack.reset();
        
        // Final garbage collection
        if self.config.enable_gc {
            self.collect_garbage()?;
        }
        
        log::info!("VM shutdown completed");
        Ok(())
    }
}

impl Default for VaaktraVm {
    fn default() -> Self {
        Self::new().expect("Failed to create default VaaktraVm")
    }
}

impl Drop for VaaktraVm {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
