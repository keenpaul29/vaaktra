//! Runtime Environment for Vāktra (वाक्त्र)
//! 
//! Provides the runtime execution environment with advanced concurrency
//! and performance features inspired by Vedic concepts.

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use parking_lot::{RwLock, Mutex};
use crossbeam::channel::{Sender, Receiver, unbounded};
use rayon::prelude::*;
use thiserror::Error;

/// Runtime errors
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Thread pool error: {0}")]
    ThreadPoolError(String),
    
    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),
    
    #[error("Resource exhaustion: {0}")]
    ResourceExhaustion(String),
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// The main runtime environment for Vāktra
/// Inspired by the concept of ब्रह्मांड (universe) containing all execution contexts
pub struct VaaktraRuntime {
    /// Thread pool for parallel execution (inspired by गण - groups)
    thread_pool: rayon::ThreadPool,
    
    /// Global symbol table
    global_symbols: Arc<RwLock<HashMap<String, RuntimeValue>>>,
    
    /// Execution contexts for different threads
    contexts: Arc<RwLock<HashMap<thread::ThreadId, ExecutionContext>>>,
    
    /// Message passing channels for actor-like concurrency
    message_channels: Arc<Mutex<HashMap<String, (Sender<RuntimeMessage>, Receiver<RuntimeMessage>)>>>,
    
    /// Runtime statistics
    stats: RuntimeStats,
    
    /// Configuration
    config: RuntimeConfig,
}

/// Runtime value types
#[derive(Debug, Clone)]
pub enum RuntimeValue {
    /// सङ्ख्या (Number)
    Sankhya(i64),
    
    /// सत्यासत्य (Boolean)
    Satyasatya(bool),
    
    /// शब्द (String)
    Shabda(String),
    
    /// सूची (List)
    Suchi(Vec<RuntimeValue>),
    
    /// निधान (Map)
    Nidhaan(HashMap<String, RuntimeValue>),
    
    /// शून्य (Void)
    Shunya,
    
    /// Function pointer
    Mantra(fn(&[RuntimeValue]) -> RuntimeResult<RuntimeValue>),
    
    /// Object instance
    Dharma(HashMap<String, RuntimeValue>),
}

/// Execution context for a thread
#[derive(Debug)]
pub struct ExecutionContext {
    /// Local symbol table
    local_symbols: HashMap<String, RuntimeValue>,
    
    /// Call stack
    call_stack: Vec<CallFrame>,
    
    /// Current instruction pointer
    instruction_pointer: usize,
    
    /// Thread-local storage
    thread_storage: HashMap<String, RuntimeValue>,
}

/// Call frame for function calls
#[derive(Debug)]
pub struct CallFrame {
    /// Function name
    function_name: String,
    
    /// Local variables
    locals: HashMap<String, RuntimeValue>,
    
    /// Return address
    return_address: usize,
    
    /// Stack pointer
    stack_pointer: usize,
}

/// Runtime message for inter-thread communication
#[derive(Debug, Clone)]
pub struct RuntimeMessage {
    pub sender: String,
    pub receiver: String,
    pub payload: RuntimeValue,
    pub message_type: MessageType,
}

/// Types of runtime messages
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    /// Function call
    Call,
    
    /// Return value
    Return,
    
    /// Error notification
    Error,
    
    /// Synchronization
    Sync,
    
    /// Custom message
    Custom(String),
}

/// Runtime statistics
#[derive(Debug, Default)]
pub struct RuntimeStats {
    pub functions_executed: usize,
    pub threads_spawned: usize,
    pub messages_sent: usize,
    pub gc_collections: usize,
    pub total_execution_time: std::time::Duration,
}

/// Runtime configuration
#[derive(Debug)]
pub struct RuntimeConfig {
    /// Maximum number of threads
    pub max_threads: usize,
    
    /// Stack size per thread
    pub stack_size: usize,
    
    /// Enable garbage collection
    pub enable_gc: bool,
    
    /// GC threshold
    pub gc_threshold: usize,
    
    /// Enable profiling
    pub enable_profiling: bool,
}

impl VaaktraRuntime {
    /// Create a new runtime with optimal configuration
    pub fn new() -> RuntimeResult<Self> {
        let num_cpus = num_cpus::get();
        
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_cpus)
            .stack_size(2 * 1024 * 1024) // 2MB stack per thread
            .thread_name(|i| format!("vaaktra-worker-{}", i))
            .build()
            .map_err(|e| RuntimeError::ThreadPoolError(e.to_string()))?;
        
        Ok(VaaktraRuntime {
            thread_pool,
            global_symbols: Arc::new(RwLock::new(HashMap::new())),
            contexts: Arc::new(RwLock::new(HashMap::new())),
            message_channels: Arc::new(Mutex::new(HashMap::new())),
            stats: RuntimeStats::default(),
            config: RuntimeConfig {
                max_threads: num_cpus * 2,
                stack_size: 2 * 1024 * 1024,
                enable_gc: true,
                gc_threshold: 1000,
                enable_profiling: false,
            },
        })
    }
    
    /// Execute a function with the given arguments
    pub fn execute_function(&mut self, name: &str, args: &[RuntimeValue]) -> RuntimeResult<RuntimeValue> {
        let start_time = std::time::Instant::now();
        
        // Get the current thread context
        let thread_id = thread::current().id();
        let mut contexts = self.contexts.write();
        let context = contexts.entry(thread_id).or_insert_with(|| ExecutionContext {
            local_symbols: HashMap::new(),
            call_stack: Vec::new(),
            instruction_pointer: 0,
            thread_storage: HashMap::new(),
        });
        
        // Create a new call frame
        let call_frame = CallFrame {
            function_name: name.to_string(),
            locals: HashMap::new(),
            return_address: context.instruction_pointer,
            stack_pointer: context.call_stack.len(),
        };
        
        context.call_stack.push(call_frame);
        
        // Look up the function in global symbols
        let global_symbols = self.global_symbols.read();
        let result = match global_symbols.get(name) {
            Some(RuntimeValue::Mantra(func)) => {
                drop(global_symbols);
                func(args)
            }
            Some(_) => Err(RuntimeError::ExecutionFailed(format!("{} is not a function", name))),
            None => Err(RuntimeError::ExecutionFailed(format!("Function {} not found", name))),
        };
        
        // Pop the call frame
        let mut contexts = self.contexts.write();
        if let Some(context) = contexts.get_mut(&thread_id) {
            context.call_stack.pop();
        }
        
        // Update statistics
        self.stats.functions_executed += 1;
        self.stats.total_execution_time += start_time.elapsed();
        
        result
    }
    
    /// Execute a function in parallel using the thread pool
    pub fn execute_parallel<F, R>(&self, tasks: Vec<F>) -> Vec<RuntimeResult<R>>
    where
        F: Fn() -> RuntimeResult<R> + Send,
        R: Send,
    {
        self.thread_pool.install(|| {
            tasks.into_par_iter().map(|task| task()).collect()
        })
    }
    
    /// Register a function in the global symbol table
    pub fn register_function(&mut self, name: String, func: fn(&[RuntimeValue]) -> RuntimeResult<RuntimeValue>) {
        let mut global_symbols = self.global_symbols.write();
        global_symbols.insert(name, RuntimeValue::Mantra(func));
    }
    
    /// Create a message channel for inter-thread communication
    pub fn create_channel(&mut self, name: String) -> RuntimeResult<()> {
        let (sender, receiver) = unbounded();
        let mut channels = self.message_channels.lock();
        channels.insert(name, (sender, receiver));
        Ok(())
    }
    
    /// Send a message through a channel
    pub fn send_message(&mut self, channel: &str, message: RuntimeMessage) -> RuntimeResult<()> {
        let channels = self.message_channels.lock();
        if let Some((sender, _)) = channels.get(channel) {
            sender.send(message)
                .map_err(|e| RuntimeError::ConcurrencyError(e.to_string()))?;
            self.stats.messages_sent += 1;
            Ok(())
        } else {
            Err(RuntimeError::ConcurrencyError(format!("Channel {} not found", channel)))
        }
    }
    
    /// Receive a message from a channel
    pub fn receive_message(&mut self, channel: &str) -> RuntimeResult<RuntimeMessage> {
        let channels = self.message_channels.lock();
        if let Some((_, receiver)) = channels.get(channel) {
            receiver.recv()
                .map_err(|e| RuntimeError::ConcurrencyError(e.to_string()))
        } else {
            Err(RuntimeError::ConcurrencyError(format!("Channel {} not found", channel)))
        }
    }
    
    /// Spawn a new thread for concurrent execution
    pub fn spawn_thread<F>(&mut self, name: String, task: F) -> RuntimeResult<thread::JoinHandle<()>>
    where
        F: FnOnce() + Send + 'static,
    {
        let handle = thread::Builder::new()
            .name(name)
            .stack_size(self.config.stack_size)
            .spawn(task)
            .map_err(|e| RuntimeError::ThreadPoolError(e.to_string()))?;
        
        self.stats.threads_spawned += 1;
        Ok(handle)
    }
    
    /// Get runtime statistics
    pub fn get_stats(&self) -> &RuntimeStats {
        &self.stats
    }
    
    /// Configure the runtime
    pub fn configure(&mut self, config: RuntimeConfig) {
        self.config = config;
    }
    
    /// Trigger garbage collection
    pub fn collect_garbage(&mut self) -> RuntimeResult<usize> {
        if !self.config.enable_gc {
            return Ok(0);
        }
        
        log::debug!("Starting garbage collection");
        
        // Simplified GC - in reality this would be much more sophisticated
        let mut collected = 0;
        
        // Clean up unused contexts
        let mut contexts = self.contexts.write();
        let active_threads: Vec<_> = contexts.keys().cloned().collect();
        
        for thread_id in active_threads {
            if let Some(context) = contexts.get_mut(&thread_id) {
                // Clean up local symbols that are no longer referenced
                let before = context.local_symbols.len();
                context.local_symbols.retain(|_, value| {
                    // Simplified: keep all values for now
                    // Real GC would do reachability analysis
                    true
                });
                collected += before - context.local_symbols.len();
            }
        }
        
        self.stats.gc_collections += 1;
        log::debug!("Garbage collection completed, collected {} objects", collected);
        
        Ok(collected)
    }
    
    /// Shutdown the runtime gracefully
    pub fn shutdown(&mut self) -> RuntimeResult<()> {
        log::info!("Shutting down Vāktra runtime");
        
        // Clear all contexts
        self.contexts.write().clear();
        
        // Clear global symbols
        self.global_symbols.write().clear();
        
        // Close all channels
        self.message_channels.lock().clear();
        
        log::info!("Runtime shutdown completed");
        Ok(())
    }
}

impl Default for VaaktraRuntime {
    fn default() -> Self {
        Self::new().expect("Failed to create default VaaktraRuntime")
    }
}

impl RuntimeValue {
    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            RuntimeValue::Sankhya(n) => n.to_string(),
            RuntimeValue::Satyasatya(b) => if *b { "सत्य".to_string() } else { "असत्य".to_string() },
            RuntimeValue::Shabda(s) => s.clone(),
            RuntimeValue::Suchi(list) => {
                let items: Vec<String> = list.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            RuntimeValue::Nidhaan(map) => {
                let items: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            RuntimeValue::Shunya => "शून्य".to_string(),
            RuntimeValue::Mantra(_) => "<mantra>".to_string(),
            RuntimeValue::Dharma(obj) => {
                let fields: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("dharma {{{}}}", fields.join(", "))
            }
        }
    }
    
    /// Check if the value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Satyasatya(b) => *b,
            RuntimeValue::Sankhya(n) => *n != 0,
            RuntimeValue::Shabda(s) => !s.is_empty(),
            RuntimeValue::Suchi(list) => !list.is_empty(),
            RuntimeValue::Nidhaan(map) => !map.is_empty(),
            RuntimeValue::Shunya => false,
            RuntimeValue::Mantra(_) => true,
            RuntimeValue::Dharma(obj) => !obj.is_empty(),
        }
    }
}

// Add num_cpus dependency to get CPU count
extern crate num_cpus;
