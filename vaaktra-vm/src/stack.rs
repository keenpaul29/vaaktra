//! VM Stack for Vāktra
//! 
//! High-performance stack implementation for the virtual machine
//! with overflow protection and efficient memory management.

use crate::{VmError, VmResult};
use vaaktra_jit::runtime::RuntimeValue;
use std::fmt;

/// Virtual machine stack
pub struct VmStack {
    /// Stack data
    data: Vec<RuntimeValue>,
    
    /// Stack pointer (index of next free slot)
    sp: usize,
    
    /// Maximum stack size
    max_size: usize,
    
    /// Call frame stack
    frames: Vec<CallFrame>,
}

/// Call frame for function calls
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Return address
    pub return_address: u32,
    
    /// Base pointer for local variables
    pub base_pointer: usize,
    
    /// Function name (for debugging)
    pub function_name: String,
    
    /// Number of local variables
    pub local_count: u16,
}

impl VmStack {
    /// Create a new VM stack
    pub fn new(max_size: usize) -> VmResult<Self> {
        Ok(VmStack {
            data: Vec::with_capacity(1024), // Start with reasonable capacity
            sp: 0,
            max_size,
            frames: Vec::new(),
        })
    }
    
    /// Push a value onto the stack
    pub fn push(&mut self, value: RuntimeValue) -> VmResult<()> {
        if self.sp >= self.max_size {
            return Err(VmError::StackOverflow("Stack overflow".to_string()));
        }
        
        // Grow the stack if needed
        if self.sp >= self.data.len() {
            self.data.resize(self.sp + 1, RuntimeValue::Shunya);
        }
        
        self.data[self.sp] = value;
        self.sp += 1;
        
        Ok(())
    }
    
    /// Pop a value from the stack
    pub fn pop(&mut self) -> VmResult<RuntimeValue> {
        if self.sp == 0 {
            return Err(VmError::ExecutionError("Stack underflow".to_string()));
        }
        
        self.sp -= 1;
        Ok(std::mem::replace(&mut self.data[self.sp], RuntimeValue::Shunya))
    }
    
    /// Peek at the top value without removing it
    pub fn peek(&self) -> VmResult<&RuntimeValue> {
        if self.sp == 0 {
            return Err(VmError::ExecutionError("Stack is empty".to_string()));
        }
        
        Ok(&self.data[self.sp - 1])
    }
    
    /// Peek at a value at offset from top
    pub fn peek_at(&self, offset: usize) -> VmResult<&RuntimeValue> {
        if offset >= self.sp {
            return Err(VmError::ExecutionError("Stack offset out of bounds".to_string()));
        }
        
        Ok(&self.data[self.sp - 1 - offset])
    }
    
    /// Duplicate the top value
    pub fn dup(&mut self) -> VmResult<()> {
        let value = self.peek()?.clone();
        self.push(value)
    }
    
    /// Swap the top two values
    pub fn swap(&mut self) -> VmResult<()> {
        if self.sp < 2 {
            return Err(VmError::ExecutionError("Not enough values to swap".to_string()));
        }
        
        self.data.swap(self.sp - 1, self.sp - 2);
        Ok(())
    }
    
    /// Get current stack size
    pub fn size(&self) -> usize {
        self.sp
    }
    
    /// Check if stack is empty
    pub fn is_empty(&self) -> bool {
        self.sp == 0
    }
    
    /// Get remaining capacity
    pub fn remaining_capacity(&self) -> usize {
        self.max_size - self.sp
    }
    
    /// Reset the stack
    pub fn reset(&mut self) {
        self.sp = 0;
        self.frames.clear();
        // Don't shrink the data vector for performance
    }
    
    /// Push a call frame
    pub fn push_frame(&mut self, frame: CallFrame) -> VmResult<()> {
        // Reserve space for local variables
        for _ in 0..frame.local_count {
            self.push(RuntimeValue::Shunya)?;
        }
        
        self.frames.push(frame);
        Ok(())
    }
    
    /// Pop a call frame
    pub fn pop_frame(&mut self) -> VmResult<CallFrame> {
        let frame = self.frames.pop()
            .ok_or_else(|| VmError::ExecutionError("No call frame to pop".to_string()))?;
        
        // Remove local variables from stack
        for _ in 0..frame.local_count {
            self.pop()?;
        }
        
        Ok(frame)
    }
    
    /// Get current call frame
    pub fn current_frame(&self) -> Option<&CallFrame> {
        self.frames.last()
    }
    
    /// Get local variable
    pub fn get_local(&self, index: u16) -> VmResult<&RuntimeValue> {
        let frame = self.current_frame()
            .ok_or_else(|| VmError::ExecutionError("No current call frame".to_string()))?;
        
        if index >= frame.local_count {
            return Err(VmError::ExecutionError(format!("Local variable index {} out of bounds", index)));
        }
        
        let stack_index = frame.base_pointer + index as usize;
        if stack_index >= self.sp {
            return Err(VmError::ExecutionError("Local variable not initialized".to_string()));
        }
        
        Ok(&self.data[stack_index])
    }
    
    /// Set local variable
    pub fn set_local(&mut self, index: u16, value: RuntimeValue) -> VmResult<()> {
        let frame = self.current_frame()
            .ok_or_else(|| VmError::ExecutionError("No current call frame".to_string()))?;
        
        if index >= frame.local_count {
            return Err(VmError::ExecutionError(format!("Local variable index {} out of bounds", index)));
        }
        
        let stack_index = frame.base_pointer + index as usize;
        if stack_index >= self.data.len() {
            self.data.resize(stack_index + 1, RuntimeValue::Shunya);
        }
        
        self.data[stack_index] = value;
        Ok(())
    }
    
    /// Get all values on the stack (for debugging)
    pub fn values(&self) -> &[RuntimeValue] {
        &self.data[..self.sp]
    }
    
    /// Get call stack depth
    pub fn call_depth(&self) -> usize {
        self.frames.len()
    }
    
    /// Print stack trace for debugging
    pub fn print_stack_trace(&self) {
        println!("=== Stack Trace ===");
        for (i, frame) in self.frames.iter().enumerate() {
            println!("  {}: {} (locals: {})", i, frame.function_name, frame.local_count);
        }
        println!("Stack size: {}/{}", self.sp, self.max_size);
    }
}

impl fmt::Debug for VmStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VmStack")
            .field("sp", &self.sp)
            .field("max_size", &self.max_size)
            .field("frames", &self.frames.len())
            .field("values", &self.values())
            .finish()
    }
}

impl fmt::Display for VmStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "VM Stack (size: {}/{})", self.sp, self.max_size)?;
        
        for (i, value) in self.values().iter().enumerate() {
            writeln!(f, "  [{}]: {}", i, value.to_string())?;
        }
        
        if !self.frames.is_empty() {
            writeln!(f, "Call Frames:")?;
            for (i, frame) in self.frames.iter().enumerate() {
                writeln!(f, "  [{}]: {} (bp: {}, locals: {})", 
                    i, frame.function_name, frame.base_pointer, frame.local_count)?;
            }
        }
        
        Ok(())
    }
}
